// Copyright 2018-2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::GenerateCode;
use derive_more::From;
use ir::Callable;
use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
};
use quote::{
    format_ident,
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

/// Generates `#[cfg(..)]` code to guard against compilation under `ink-as-dependency`.
#[derive(From)]
pub struct CrossCallingConflictCfg<'a> {
    contract: &'a ir::Contract,
}

impl GenerateCode for CrossCallingConflictCfg<'_> {
    fn generate_code(&self) -> TokenStream2 {
        if self.contract.config().is_compile_as_dependency_enabled() {
            return quote! { #[cfg(feature = "__ink_DO_NOT_COMPILE")] }
        }
        quote! { #[cfg(not(feature = "ink-as-dependency"))] }
    }
}

/// Generates code for using this ink! contract as a dependency.
#[derive(From)]
pub struct CrossCalling<'a> {
    contract: &'a ir::Contract,
}

impl GenerateCode for CrossCalling<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let storage = self.generate_storage();
        let standard_impls = self.generate_standard_impls();
        let call_forwarder = self.generate_call_forwarder();
        quote! {
            #storage
            #standard_impls
            #call_forwarder
        }
    }
}

impl CrossCalling<'_> {
    /// Generates code for conditionally compiling code only if the contract
    /// is compiled as dependency.
    fn generate_cfg(&self) -> Option<TokenStream2> {
        if self.contract.config().is_compile_as_dependency_enabled() {
            return None
        }
        Some(quote! {
            #[cfg(feature = "ink-as-dependency")]
        })
    }

    /// Generates code for the ink! storage struct for cross-calling purposes.
    ///
    /// # Note
    ///
    /// This always consists of a single `AccountId` and can be viewed as a
    /// reference to a live smart contract instance of the same type. It will
    /// forward all calls via ink!'s provided cross-calling infrastructure
    /// automatically over the chain.
    fn generate_storage(&self) -> TokenStream2 {
        let cfg = self.generate_cfg();
        let storage = self.contract.module().storage();
        let span = storage.span();
        let ident = storage.ident();
        let attrs = storage.attrs();
        quote_spanned!(span =>
            #cfg
            #( #attrs )*
            #[derive(
                Clone,
                Debug,
                ::scale::Encode,
                ::scale::Decode,
                ::ink_core::storage2::traits::SpreadLayout,
                ::ink_core::storage2::traits::PackedLayout,
            )]
            #[cfg_attr(
                feature = "std",
                derive(
                    ::scale_info::TypeInfo,
                    ::ink_core::storage2::traits::StorageLayout,
                )
            )]
            pub struct #ident {
                account_id: AccountId,
            }
        )
    }

    /// Generates code for the trait implementations required to make the
    /// generated ink! storage struct for cross-calling work out of the box
    /// for the cross-calling infrastructure.
    fn generate_standard_impls(&self) -> TokenStream2 {
        let cfg = self.generate_cfg();
        let ident = self.contract.module().storage().ident();
        quote! {
            #cfg
            const _: () = {
                impl ::ink_core::env::call::FromAccountId<EnvTypes> for #ident {
                    #[inline]
                    fn from_account_id(account_id: AccountId) -> Self {
                        Self { account_id }
                    }
                }

                impl ::ink_lang::ToAccountId<EnvTypes> for #ident {
                    #[inline]
                    fn to_account_id(&self) -> AccountId {
                        self.account_id
                    }
                }
            };
        }
    }

    /// Returns the identifier of a forwarded ink! message.
    ///
    /// These need to be dependending on the message's selector instead of their
    /// display name to disambiguate especially in case of trait implementations.
    ///
    /// We use the unique ID of the selector because it is simpler to convert to
    /// an identifier.
    fn forwarded_message_ident(callable: ir::CallableWithSelector<ir::Message>) -> Ident {
        format_ident!("__ink_message_{}", callable.composed_selector().unique_id())
    }

    /// Builds up the [`ink_core::env::call::ArgumentList`] type structure for the given types.
    fn generate_arg_list<'a, Args>(args: Args) -> TokenStream2
    where
        Args: IntoIterator<Item = &'a syn::Type>,
        <Args as IntoIterator>::IntoIter: DoubleEndedIterator,
    {
        args.into_iter().rev().fold(
            quote! { ::ink_core::env::call::EmptyArgumentList },
            |rest, arg| quote! {
                ::ink_core::env::call::ArgumentList<::ink_core::env::call::Argument<#arg>, #rest>
            }
        )
    }

    /// Generates code for call forwarding for the given message and its selector.
    fn generate_call_forwarding_for_message(
        callable: ir::CallableWithSelector<ir::Message>,
    ) -> TokenStream2 {
        let message = callable.callable();
        let span = message.span();
        let ident = Self::forwarded_message_ident(callable);
        let composed_selector = callable.composed_selector().as_bytes().to_owned();
        let attrs = message.attrs();
        let input_bindings = message
            .inputs()
            .enumerate()
            .map(|(n, _)| format_ident!("__ink_binding_{}", n))
            .collect::<Vec<_>>();
        let input_types = message
            .inputs()
            .map(|pat_type| &*pat_type.ty)
            .collect::<Vec<_>>();
        let arg_list = Self::generate_arg_list(input_types.iter().cloned());
        let output = message.output();
        let output_param =
            output.map_or_else(|| quote! { () }, |output| quote! { #output });
        let output_sig = output.map_or_else(
            || quote! { () },
            |output| quote! { ::ink_core::env::call::ReturnType<#output> },
        );
        let instantiate_ident = match output {
            Some(_) => format_ident!("eval"),
            None => format_ident!("invoke"),
        };
        quote_spanned!(span=>
            #( #attrs )*
            #[inline]
            pub fn #ident(
                self,
                #( #input_bindings : #input_types ),*
            ) -> ::ink_core::env::call::CallBuilder<
                EnvTypes, #arg_list, #output_sig, ::ink_core::env::call::state::Sealed
            > {
                ::ink_core::env::call::CallParams::<EnvTypes, ::ink_core::env::call::EmptyArgumentList, #output_param>::#instantiate_ident(
                    ::ink_lang::ToAccountId::to_account_id(self.contract),
                    ::ink_core::env::call::Selector::new([ #( #composed_selector ),* ]),
                )
                #(
                    .push_arg(#input_bindings)
                )*
                .seal()
            }
        )
    }

    /// Returns an iterator over all ink! messages of the contract and their selectors.
    fn contract_messages(
        &self,
    ) -> impl Iterator<Item = ir::CallableWithSelector<ir::Message>> {
        self.contract
            .module()
            .impls()
            .map(|item_impl| item_impl.iter_messages())
            .flatten()
    }

    /// Returns the identifier for the generated call forwarder utility.
    fn call_forwarder_ident() -> Ident {
        format_ident!("__ink_CallForwarder")
    }

    /// Generates code for the call forwarder utility struct.
    fn generate_call_forwarder(&self) -> TokenStream2 {
        let forwarder_ident = Self::call_forwarder_ident();
        let storage_ident = self.contract.module().storage().ident();
        let ref_self_messages = self
            .contract_messages()
            .filter(|cws| cws.callable().receiver().is_ref())
            .map(Self::generate_call_forwarding_for_message);
        let ref_mut_self_messages = self
            .contract_messages()
            .filter(|cws| cws.callable().receiver().is_ref_mut())
            .map(Self::generate_call_forwarding_for_message);
        let cfg = self.generate_cfg();

        quote! {
            #cfg
            const _: () = {
                impl<'a> ::ink_lang::ForwardCall for &'a #storage_ident {
                    type Forwarder = #forwarder_ident<&'a #storage_ident>;

                    #[inline]
                    fn call(self) -> Self::Forwarder {
                        #forwarder_ident { contract: self }
                    }
                }

                impl<'a> ::ink_lang::ForwardCallMut for &'a mut #storage_ident {
                    type Forwarder = #forwarder_ident<&'a mut #storage_ident>;

                    #[inline]
                    fn call_mut(self) -> Self::Forwarder {
                        #forwarder_ident { contract: self }
                    }
                }

                // Forwards contract messages to the chain.
                pub struct #forwarder_ident<T> {
                    contract: T,
                }

                impl<'a> #forwarder_ident<&'a #storage_ident> {
                    #( #ref_self_messages )*
                }

                impl<'a> #forwarder_ident<&'a mut #storage_ident> {
                    #( #ref_mut_self_messages )*
                }
            };
        }
    }
}
