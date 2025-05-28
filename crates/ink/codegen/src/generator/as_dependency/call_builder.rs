// Copyright (C) Use Ink (UK) Ltd.
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

use derive_more::From;
use ir::Callable;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

#[cfg(any(ink_abi = "sol", ink_abi = "all"))]
use crate::generator::sol;
use crate::{
    generator,
    GenerateCode,
};

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
    /// use the [`ink::TraitCallBuilder`] trait implementation.
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
                ::ink::storage::traits::StorageLayout,
            ))]
            #[derive(
                ::core::fmt::Debug,
                ::core::hash::Hash,
                ::core::cmp::PartialEq,
                ::core::cmp::Eq,
                ::core::clone::Clone,
            )]
            #[::ink::scale_derive(Encode, Decode, TypeInfo)]
            pub struct #cb_ident {
                addr: ::ink::Address,
            }

            const _: () = {
                impl ::ink::codegen::ContractCallBuilder for #storage_ident {
                    type Type = #cb_ident;
                }

                impl ::ink::env::ContractEnv for #cb_ident {
                    type Env = <#storage_ident as ::ink::env::ContractEnv>::Env;
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
            impl ::ink::env::call::FromAddr for #cb_ident {
                #[inline]
                fn from_addr(addr: ::ink::Address) -> Self {
                    Self { addr }
                }
            }

            impl ::ink::ToAddr for #cb_ident {
                #[inline]
                fn to_addr(&self) -> ::ink::Address {
                    <::ink::Address as ::core::clone::Clone>::clone(&self.addr)
                }
            }

            impl ::core::convert::AsRef<::ink::Address> for #cb_ident {
                fn as_ref(&self) -> &::ink::Address {
                    &self.addr
                }
            }

            impl ::core::convert::AsMut<::ink::Address> for #cb_ident {
                fn as_mut(&mut self) -> &mut ::ink::Address {
                    &mut self.addr
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
        let trait_info_id = generator::generate_reference_to_trait_info(span, trait_path);
        quote_spanned!(span=>
            #[doc(hidden)]
            impl ::ink::codegen::TraitCallForwarderFor<{#trait_info_id}> for #cb_ident {
                type Forwarder = <<Self as #trait_path>::__ink_TraitInfo as ::ink::codegen::TraitCallForwarder>::Forwarder;

                #[inline]
                fn forward(&self) -> &Self::Forwarder {
                    // SAFETY:
                    //
                    // We convert from a shared reference to a type that thinly wraps
                    // only an `AccountId` to a shared reference to another type of which
                    // we know that it also thinly wraps an `AccountId`.
                    // Furthermore both types use `repr(transparent)`.
                    // todo
                    unsafe {
                        &*(&self.addr as *const ::ink::Address as *const Self::Forwarder)
                    }
                }

                #[inline]
                fn forward_mut(&mut self) -> &mut Self::Forwarder {
                    // SAFETY:
                    //
                    // We convert from an exclusive reference to a type that thinly wraps
                    // only an `AccountId` to an exclusive reference to another type of which
                    // we know that it also thinly wraps an `AccountId`.
                    // Furthermore both types use `repr(transparent)`.
                    unsafe {
                        &mut *(&mut self.addr as *mut ::ink::Address as *mut Self::Forwarder)
                    }
                }

                #[inline]
                fn build(&self) -> &<Self::Forwarder as ::ink::codegen::TraitCallBuilder>::Builder {
                    <_ as ::ink::codegen::TraitCallBuilder>::call(
                        <Self as ::ink::codegen::TraitCallForwarderFor<{#trait_info_id}>>::forward(self)
                    )
                }

                #[inline]
                fn build_mut(&mut self)
                    -> &mut <Self::Forwarder as ::ink::codegen::TraitCallBuilder>::Builder
                {
                    <_ as ::ink::codegen::TraitCallBuilder>::call_mut(
                        <Self as ::ink::codegen::TraitCallForwarderFor<{#trait_info_id}>>::forward_mut(self)
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
                type __ink_TraitInfo = <::ink::reflect::TraitDefinitionRegistry<Environment>
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
        let cfg_attrs = message.get_cfg_attrs(span);
        let trait_info_id = generator::generate_reference_to_trait_info(span, trait_path);
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
        let attrs = self
            .contract
            .config()
            .whitelisted_attributes()
            .filter_attr(message.attrs().to_vec());
        quote_spanned!(span=>
            #( #cfg_attrs )*
            type #output_ident = <<<
                Self
                as ::ink::codegen::TraitCallForwarderFor<{#trait_info_id}>>::Forwarder
                as ::ink::codegen::TraitCallBuilder>::Builder
                as #trait_path>::#output_ident;

            #[inline]
            #( #attrs )*
            fn #message_ident(
                & #mut_token self
                #( , #input_bindings: #input_types )*
            ) -> Self::#output_ident {
                <_ as #trait_path>::#message_ident(
                    <Self as ::ink::codegen::TraitCallForwarderFor<{#trait_info_id}>>::#build_cmd(self)
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
            .filter(|impl_block| impl_block.trait_path().is_none())
            .map(|impl_block| self.generate_call_builder_inherent_impl(impl_block))
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
        let attrs = self
            .contract
            .config()
            .whitelisted_attributes()
            .filter_attr(message.attrs().to_vec());
        let input_bindings = generator::input_bindings(callable.inputs());
        let input_types = generator::input_types(message.inputs());
        let mut_tok = callable.receiver().is_ref_mut().then(|| quote! { mut });
        let return_type = message
            .output()
            .map(quote::ToTokens::to_token_stream)
            .unwrap_or_else(|| quote::quote! { () });
        let output_span = return_type.span();

        let mut call_builders = Vec::new();

        #[cfg(not(ink_abi = "sol"))]
        {
            let selector = message.composed_selector();
            let selector_bytes = selector.hex_lits();
            let arg_list = generator::generate_argument_list(
                input_types.iter().cloned(),
                quote!(::ink::reflect::ScaleEncoding),
            );
            let output_type = quote_spanned!(output_span=>
                ::ink::env::call::CallBuilder<
                    Environment,
                    ::ink::env::call::utils::Set< ::ink::env::call::Call >,
                    ::ink::env::call::utils::Set< ::ink::env::call::ExecutionInput<#arg_list, ::ink::reflect::ScaleEncoding> >,
                    ::ink::env::call::utils::Set< ::ink::env::call::utils::ReturnType<#return_type> >,
                >
            );

            let call_builder = quote_spanned!(span=>
                #( #attrs )*
                #[allow(clippy::type_complexity)]
                #[inline]
                pub fn #message_ident(
                    & #mut_tok self
                    #( , #input_bindings : #input_types )*
                ) -> #output_type {
                    ::ink::env::call::build_call::<Environment>()
                        .call(::ink::ToAddr::to_addr(self))
                        .exec_input(
                            ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new([ #( #selector_bytes ),* ])
                            )
                            #(
                                .push_arg(#input_bindings)
                            )*
                        )
                        .returns::<#return_type>()
                }
            );
            call_builders.push(call_builder);
        }

        #[cfg(any(ink_abi = "sol", ink_abi = "all"))]
        {
            // If ABI is all, we generate a second message signature with a "_sol"
            // postfix. Otherwise, we use the same name.
            let sol_message_ident = if cfg!(ink_abi = "all") {
                format_ident!("{}_sol", message_ident)
            } else {
                message_ident.clone()
            };
            let selector_bytes = sol::utils::selector(&message);
            let arg_list = generator::generate_argument_list(
                input_types.iter().cloned(),
                quote!(::ink::reflect::SolEncoding),
            );
            let output_type = quote_spanned!(output_span=>
                ::ink::env::call::CallBuilder<
                    Environment,
                    ::ink::env::call::utils::Set< ::ink::env::call::Call >,
                    ::ink::env::call::utils::Set< ::ink::env::call::ExecutionInput<#arg_list, ::ink::reflect::SolEncoding> >,
                    ::ink::env::call::utils::Set< ::ink::env::call::utils::ReturnType<#return_type> >,
                >
            );

            let call_builder = quote_spanned!(span=>
                #( #attrs )*
                #[allow(clippy::type_complexity)]
                #[inline]
                pub fn #sol_message_ident (
                    & #mut_tok self
                    #( , #input_bindings : #input_types )*
                ) -> #output_type {
                    ::ink::env::call::build_call_solidity::<Environment>()
                        .call(::ink::ToAddr::to_addr(self))
                        .exec_input(
                            ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new(#selector_bytes)
                            )
                            #(
                                .push_arg(#input_bindings)
                            )*
                        )
                        .returns::<#return_type>()
                }
            );
            call_builders.push(call_builder);
        }

        quote_spanned!(span=>
            #( #call_builders )*
        )
    }
}
