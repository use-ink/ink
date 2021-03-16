// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

//! Generates the definition and implementations for the ink! trait definition resolver.
//!
//! This is used in order code in order to resolve an ink! trait definition to a proper
//! Rust type that can be used in the Rust type system and used as instance. This is
//! for example required in order to cross-call a trait based contract implementation or
//! to allow for contract references in arguments that implement a trait by definition
//! of the API.

use super::TraitDefinition;
use heck::CamelCase as _;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};

impl<'a> TraitDefinition<'a> {
    pub(super) fn generate_trait_concretizer(&self) -> TokenStream2 {
        let span = self.trait_def.span();
        let hash = self.trait_def.verify_hash();
        let ident = self.trait_def.ident();
        let concrete_implementer_ident = format_ident!(
            "__ink_ConcreteImplementer{}_0x{:X}{:X}{:X}{:X}",
            ident,
            hash[0],
            hash[1],
            hash[2],
            hash[3]
        );
        let verify_hash_id =
            u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]) as usize;
        let constructors_never_call = self
            .trait_def
            .iter_items()
            .map(|(item, _)| item)
            .flat_map(ir::InkTraitItem::filter_map_constructor)
            .map(Self::generate_for_constructor_never_call);
        let messages_never_call = self
            .trait_def
            .iter_items()
            .map(|(item, _)| item)
            .flat_map(ir::InkTraitItem::filter_map_message)
            .map(Self::generate_for_message_never_call);
        quote_spanned!(span =>
            const _: () = {
                /// A universal concrete implementer of the ink! trait definition.
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                pub struct #concrete_implementer_ident<E>
                where
                    E: ::ink_env::Environment,
                {
                    account_id: <E as ::ink_env::Environment>::AccountId,
                }

                impl<E> ::ink_env::call::FromAccountId<E> for #concrete_implementer_ident<E>
                where
                    E: ::ink_env::Environment,
                {
                    #[inline]
                    fn from_account_id(account_id: <E as ::ink_env::Environment>::AccountId) -> Self {
                        Self { account_id }
                    }
                }

                impl<E> ::ink_lang::ToAccountId<E> for #concrete_implementer_ident<E>
                where
                    E: ::ink_env::Environment,
                    <E as ::ink_env::Environment>::AccountId: Clone,
                {
                    #[inline]
                    fn to_account_id(&self) -> <E as ::ink_env::Environment>::AccountId {
                        self.account_id.clone()
                    }
                }

                impl<E> ::core::clone::Clone for #concrete_implementer_ident<E>
                where
                    E: ::ink_env::Environment,
                    <E as ::ink_env::Environment>::AccountId: Clone,
                {
                    fn clone(&self) -> Self {
                        Self { account_id: self.account_id.clone() }
                    }
                }

                impl<E> #ident for ::ink_lang::ConcreteImplementers<E>
                where
                    E: ::ink_env::Environment,
                {
                    #[doc(hidden)]
                    #[allow(non_camel_case_types)]
                    type __ink_Checksum = [(); #verify_hash_id];

                    #[doc(hidden)]
                    #[allow(non_camel_case_types)]
                    type __ink_ConcreteImplementer = #concrete_implementer_ident<E>;

                    #(#constructors_never_call)*
                    #(#messages_never_call)*
                }
            };
        )
    }

    fn generate_for_constructor_never_call(
        constructor: ir::InkTraitConstructor<'a>,
    ) -> TokenStream2 {
        let span = constructor.span();
        let attrs = constructor.attrs();
        let sig = constructor.sig();
        let ident = &sig.ident;
        let output_ident = format_ident!("{}Out", ident.to_string().to_camel_case());
        let input_bindings = sig
            .inputs
            .iter()
            .enumerate()
            .map(|(n, fn_arg)| {
                match fn_arg {
                    syn::FnArg::Typed(pat_type) => {
                        let ident = format_ident!("__ink_binding_{}", n);
                        let ty = &pat_type.ty;
                        quote! { #ident : #ty }
                    }
                    syn::FnArg::Receiver(receiver) => quote! { #receiver },
                }
            })
            .collect::<Vec<_>>();
        let linker_error_ident =
            format_ident!("{}", "__ink_enforce_error_for_constructor");
        quote_spanned!(span =>
            /// Output type of the respective trait constructor.
            type #output_ident = ::ink_lang::NeverReturns;

            #(#attrs)*
            fn #ident(#(#input_bindings),*) -> Self::#output_ident {
                extern {
                    fn #linker_error_ident() -> !;
                }
                unsafe { #linker_error_ident() }
            }
        )
    }

    fn generate_for_message_never_call(message: ir::InkTraitMessage<'a>) -> TokenStream2 {
        let span = message.span();
        let attrs = message.attrs();
        let sig = message.sig();
        let ident = &sig.ident;
        let output = match &sig.output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, ty) => quote! { #ty },
        };
        let output_ident = format_ident!("{}Out", ident.to_string().to_camel_case());
        let input_bindings = sig
            .inputs
            .iter()
            .enumerate()
            .map(|(n, fn_arg)| {
                match fn_arg {
                    syn::FnArg::Typed(pat_type) => {
                        let ident = format_ident!("__ink_binding_{}", n);
                        let ty = &pat_type.ty;
                        quote! { #ident : #ty }
                    }
                    syn::FnArg::Receiver(receiver) => quote! { #receiver },
                }
            })
            .collect::<Vec<_>>();
        let linker_error_ident = format_ident!("{}", "__ink_enforce_error_for_message");
        quote_spanned!(span =>
            /// Output type of the respective trait constructor.
            type #output_ident = #output;

            #(#attrs)*
            fn #ident(#(#input_bindings),*) -> Self::#output_ident {
                extern {
                    fn #linker_error_ident() -> !;
                }
                unsafe { #linker_error_ident() }
            }
        )
    }
}
