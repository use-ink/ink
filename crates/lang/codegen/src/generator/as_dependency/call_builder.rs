// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use crate::{
    generator,
    GenerateCode,
};
use derive_more::From;
use ir::Callable;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

/// Generates code for the call builder of the ink! smart contract.
///
/// The call builder is the entity that builds up calls for calling of other
/// smart contract on-chain in a type safe way.
/// It implements all ink! traits that the associated ink! smart contract implements
/// so that their underlying implementation directly calls the respective ink!
/// trait implementation on-chain.
///
/// The ink! call builder of a smart contract is directly used by the storage
/// type of the smart contract itself as well by other entities that use the
/// smart contract via long-hand calling notation to incrementally build up calls.
#[derive(From)]
pub struct CallBuilder<'a> {
    contract: &'a ir::Contract,
}

impl GenerateCode for CallBuilder<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let call_builder_struct = self.generate_struct();
        let auxiliary_trait_impls = self.generate_auxiliary_trait_impls();
        let call_builder_impls = self.generate_call_forwarder_impls();
        let call_builder_inherent_impls = self.generate_call_builder_inherent_impls();
        quote! {
            const _: () = {
                #call_builder_struct
                #auxiliary_trait_impls
                #call_builder_impls
                #call_builder_inherent_impls
            };
        }
    }
}

impl CallBuilder<'_> {
    /// Returns the identifier of the generated ink! call builder struct.
    ///
    /// # Note
    ///
    /// This identifier must not be used outside of the generated `const`
    /// block in which the call builder type is going to be defined.
    /// In order to refer to the call builder of an ink! smart contract
    /// use the [`ink_lang::TraitCallBuilder`] trait implementation.
    fn call_builder_ident() -> syn::Ident {
        format_ident!("CallBuilder")
    }

    fn generate_struct(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let cb_ident = Self::call_builder_ident();
        quote_spanned!(span=>
            /// The ink! smart contract's call builder.
            ///
            /// Implements the underlying on-chain calling of the ink! smart contract
            /// messages and trait implementations in a type safe way.
            #[repr(transparent)]
            #[cfg_attr(feature = "std", derive(
                ::scale_info::TypeInfo,
                ::ink_storage::traits::StorageLayout,
            ))]
            #[derive(
                ::core::fmt::Debug,
                ::ink_storage::traits::SpreadLayout,
                ::ink_storage::traits::PackedLayout,
                ::scale::Encode,
                ::scale::Decode,
                ::core::hash::Hash,
                ::core::cmp::PartialEq,
                ::core::cmp::Eq,
                ::core::clone::Clone,
            )]
            pub struct #cb_ident {
                account_id: AccountId,
            }

            const _: () = {
                impl ::ink_lang::codegen::ContractCallBuilder for #storage_ident {
                    type Type = #cb_ident;
                }

                impl ::ink_lang::reflect::ContractEnv for #cb_ident {
                    type Env = <#storage_ident as ::ink_lang::reflect::ContractEnv>::Env;
                }
            };
        )
    }

    /// Generates some ink! specific auxiliary trait implementations for the
    /// smart contract call builder type.
    ///
    /// These are required to properly interoperate with the call builder.
    fn generate_auxiliary_trait_impls(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let cb_ident = Self::call_builder_ident();
        quote_spanned!(span=>
            impl ::ink_env::call::FromAccountId<Environment> for #cb_ident {
                #[inline]
                fn from_account_id(account_id: AccountId) -> Self {
                    Self { account_id }
                }
            }

            impl ::ink_lang::ToAccountId<Environment> for #cb_ident {
                #[inline]
                fn to_account_id(&self) -> AccountId {
                    <AccountId as ::core::clone::Clone>::clone(&self.account_id)
                }
            }
        )
    }

    /// Generate the `TraitCallForwarder` trait implementations for the call builder
    /// for every ink! trait implemented by the associated ink! smart contract.
    ///
    /// These call forwarder trait implementations are used to dispatch to the global
    /// call builder for the respective ink! trait definition that is being called.
    /// The call builder only forwards the actual calls to those global call builders
    /// and does not have its own calling logic.
    fn generate_call_forwarder_impls(&self) -> TokenStream2 {
        self.contract
            .module()
            .impls()
            .filter_map(|impl_block| {
                // We are only interested in ink! trait implementation block.
                impl_block.trait_path().map(|trait_path| {
                    self.generate_code_for_trait_impl(trait_path, impl_block)
                })
            })
            .collect()
    }

    /// Generates code required by the ink! call builder of an ink! smart contract
    /// for a single ink! trait definition that the contract implements.
    fn generate_code_for_trait_impl(
        &self,
        trait_path: &syn::Path,
        impl_block: &ir::ItemImpl,
    ) -> TokenStream2 {
        let call_forwarder_impl =
            self.generate_call_forwarder_for_trait_impl(trait_path, impl_block);
        let ink_trait_impl = self.generate_ink_trait_impl(trait_path, impl_block);
        quote! {
            #call_forwarder_impl
            #ink_trait_impl
        }
    }

    /// Generates code for a single ink! trait implementation to forward calls for
    /// the associated ink! smart contract call builder.
    fn generate_call_forwarder_for_trait_impl(
        &self,
        trait_path: &syn::Path,
        impl_block: &ir::ItemImpl,
    ) -> TokenStream2 {
        let span = impl_block.span();
        let cb_ident = Self::call_builder_ident();
        let trait_info = generator::generate_reference_to_trait_info(span, trait_path);
        quote_spanned!(span=>
            #[doc(hidden)]
            impl ::ink_lang::codegen::TraitCallForwarderFor<#trait_info> for #cb_ident {
                type Forwarder = <<Self as #trait_path>::__ink_TraitInfo as ::ink_lang::codegen::TraitCallForwarder>::Forwarder;

                #[inline]
                fn forward(&self) -> &Self::Forwarder {
                    // SAFETY:
                    //
                    // We convert from a shared reference to a type that thinly wraps
                    // only an `AccountId` to a shared reference to another type of which
                    // we know that it also thinly wraps an `AccountId`.
                    // Furthermore both types use `repr(transparent)`.
                    unsafe {
                        &*(&self.account_id as *const AccountId as *const Self::Forwarder)
                    }
                }

                #[inline]
                fn forward_mut(&mut self) -> &mut Self::Forwarder {
                    // SAFETY:
                    //
                    // We convert from a exclusive reference to a type that thinly wraps
                    // only an `AccountId` to a exclusive reference to another type of which
                    // we know that it also thinly wraps an `AccountId`.
                    // Furthermore both types use `repr(transparent)`.
                    unsafe {
                        &mut *(&mut self.account_id as *mut AccountId as *mut Self::Forwarder)
                    }
                }

                #[inline]
                fn build(&self) -> &<Self::Forwarder as ::ink_lang::codegen::TraitCallBuilder>::Builder {
                    <_ as ::ink_lang::codegen::TraitCallBuilder>::call(
                        <Self as ::ink_lang::codegen::TraitCallForwarderFor<#trait_info>>::forward(self)
                    )
                }

                #[inline]
                fn build_mut(&mut self)
                    -> &mut <Self::Forwarder as ::ink_lang::codegen::TraitCallBuilder>::Builder
                {
                    <_ as ::ink_lang::codegen::TraitCallBuilder>::call_mut(
                        <Self as ::ink_lang::codegen::TraitCallForwarderFor<#trait_info>>::forward_mut(self)
                    )
                }
            }
        )
    }

    /// Generates the actual ink! trait implementation for the generated call builder.
    fn generate_ink_trait_impl(
        &self,
        trait_path: &syn::Path,
        impl_block: &ir::ItemImpl,
    ) -> TokenStream2 {
        let span = impl_block.span();
        let cb_ident = Self::call_builder_ident();
        let messages = impl_block
            .iter_messages()
            .map(|message| self.generate_ink_trait_impl_for_message(trait_path, message));
        quote_spanned!(span=>
            impl #trait_path for #cb_ident {
                type __ink_TraitInfo = <::ink_lang::reflect::TraitDefinitionRegistry<Environment>
                    as #trait_path>::__ink_TraitInfo;

                #( #messages )*
            }
        )
    }

    /// Generates the code for the ink! trait implementation of the call builder
    /// of a single ink! trait message and its associated output type.
    fn generate_ink_trait_impl_for_message(
        &self,
        trait_path: &syn::Path,
        message: ir::CallableWithSelector<ir::Message>,
    ) -> TokenStream2 {
        use ir::Callable as _;
        let span = message.span();
        let message_ident = message.ident();
        let output_ident = generator::output_ident(message_ident);
        let trait_info = generator::generate_reference_to_trait_info(span, trait_path);
        let (input_bindings, input_types): (Vec<_>, Vec<_>) = message
            .callable()
            .inputs()
            .map(|input| (&input.pat, &input.ty))
            .unzip();
        let mut_token = message
            .receiver()
            .is_ref_mut()
            .then(|| Some(quote! { mut }));
        let build_cmd = match message.receiver() {
            ir::Receiver::Ref => quote! { build },
            ir::Receiver::RefMut => quote! { build_mut },
        };
        let attrs = message.attrs();
        quote_spanned!(span=>
            type #output_ident = <<<
                Self
                as ::ink_lang::codegen::TraitCallForwarderFor<#trait_info>>::Forwarder
                as ::ink_lang::codegen::TraitCallBuilder>::Builder
                as #trait_path>::#output_ident;

            #[inline]
            #( #attrs )*
            fn #message_ident(
                & #mut_token self
                #( , #input_bindings: #input_types )*
            ) -> Self::#output_ident {
                <_ as #trait_path>::#message_ident(
                    <Self as ::ink_lang::codegen::TraitCallForwarderFor<#trait_info>>::#build_cmd(self)
                    #( , #input_bindings )*
                )
            }
        )
    }

    /// Generate call builder code for all ink! inherent ink! implementation blocks.
    ///
    /// # Note
    ///
    /// This does not provide implementations for ink! constructors as they
    /// do not have a short-hand notations as their messages counterparts.
    fn generate_call_builder_inherent_impls(&self) -> TokenStream2 {
        self.contract
            .module()
            .impls()
            .filter_map(|impl_block| {
                impl_block
                    .trait_path()
                    .is_none()
                    .then(|| self.generate_call_builder_inherent_impl(impl_block))
            })
            .collect()
    }

    /// Generate call builder code for a single inherent ink! implementation block.
    ///
    /// # Note
    ///
    /// Unlike as with ink! trait implementation blocks we do not have to generate
    /// associate `*Output` types, ink! trait validating implementation blocks or
    /// trait forwarder implementations. Instead we build the calls directly.
    fn generate_call_builder_inherent_impl(
        &self,
        impl_block: &ir::ItemImpl,
    ) -> TokenStream2 {
        let span = impl_block.span();
        let cb_ident = Self::call_builder_ident();
        let messages = impl_block
            .iter_messages()
            .map(|message| self.generate_call_builder_inherent_impl_for_message(message));
        quote_spanned!(span=>
            impl #cb_ident {
                #( #messages )*
            }
        )
    }

    /// Generate call builder code for a single inherent ink! message.
    ///
    /// # Note
    ///
    /// Unlike with ink! trait messages the call builder implements the call
    /// building directly and does not forward to a trait call builder.
    fn generate_call_builder_inherent_impl_for_message(
        &self,
        message: ir::CallableWithSelector<ir::Message>,
    ) -> TokenStream2 {
        let span = message.span();
        let callable = message.callable();
        let message_ident = message.ident();
        let attrs = message.attrs();
        let selector = message.composed_selector();
        let selector_bytes = selector.hex_lits();
        let input_bindings = generator::input_bindings(callable.inputs());
        let input_types = generator::input_types(message.inputs());
        let arg_list = generator::generate_argument_list(input_types.iter().cloned());
        let mut_tok = callable.receiver().is_ref_mut().then(|| quote! { mut });
        let output = message.output();
        let output_sig = output.map_or_else(
            || quote! { () },
            |output| quote! { ::ink_env::call::utils::ReturnType<#output> },
        );
        let output_span = output.span();
        let output_type = quote_spanned!(output_span=>
            ::ink_env::call::CallBuilder<
                Environment,
                ::ink_env::call::utils::Set< <Environment as ::ink_env::Environment>::AccountId >,
                ::ink_env::call::utils::Unset< ::core::primitive::u64 >,
                ::ink_env::call::utils::Unset< <Environment as ::ink_env::Environment>::Balance >,
                ::ink_env::call::utils::Set< ::ink_env::call::ExecutionInput<#arg_list> >,
                ::ink_env::call::utils::Set<#output_sig>,
            >
        );
        quote_spanned!(span=>
            #( #attrs )*
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn #message_ident(
                & #mut_tok self
                #( , #input_bindings : #input_types )*
            ) -> #output_type {
                ::ink_env::call::build_call::<Environment>()
                    .callee(::ink_lang::ToAccountId::to_account_id(self))
                    .exec_input(
                        ::ink_env::call::ExecutionInput::new(
                            ::ink_env::call::Selector::new([ #( #selector_bytes ),* ])
                        )
                        #(
                            .push_arg(#input_bindings)
                        )*
                    )
                    .returns::<#output_sig>()
            }
        )
    }
}
