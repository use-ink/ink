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

use crate::{
    generator,
    GenerateCode,
    GenerateCodeUsing as _,
};
use derive_more::From;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

/// Generates code for the ink! event structs of the contract.
#[derive(From)]
pub struct Events<'a> {
    contract: &'a ir::Contract,
}

impl AsRef<ir::Contract> for Events<'_> {
    fn as_ref(&self) -> &ir::Contract {
        self.contract
    }
}

impl GenerateCode for Events<'_> {
    fn generate_code(&self) -> TokenStream2 {
        if self.contract.module().events().next().is_none() {
            // Generate no code in case there are no event definitions.
            return TokenStream2::new()
        }
        let emit_event_trait_impl = self.generate_emit_event_trait_impl();
        let event_base = self.generate_event_base();
        let topics_impls = self.generate_topics_impls();
        let event_structs = self.generate_event_structs();
        quote! {
            #emit_event_trait_impl
            #event_base
            #( #event_structs )*
            #( #topics_impls )*
        }
    }
}

impl Events<'_> {
    /// Used to allow emitting user defined events directly instead of converting
    /// them first into the automatically generated base trait of the contract.
    fn generate_emit_event_trait_impl(&self) -> TokenStream2 {
        let storage_ident = &self.contract.module().storage().ident();
        quote! {
            const _: () = {
                impl<'a> ::ink_lang::EmitEvent<#storage_ident> for ::ink_lang::EnvAccess<'a, EnvTypes> {
                    fn emit_event<E>(self, event: E)
                    where
                        E: Into<<#storage_ident as ::ink_lang::BaseEvent>::Type>,
                    {
                        ::ink_core::env::emit_event::<
                            EnvTypes,
                            <#storage_ident as ::ink_lang::BaseEvent>::Type
                        >(event.into());
                    }
                }
            };
        }
    }

    /// Generates the base event enum that comprises all user defined events.
    /// All emitted events are converted into a variant of this enum before being
    /// serialized and emitted to apply their unique event discriminant (ID).
    fn generate_event_base(&self) -> TokenStream2 {
        let storage_ident = &self.contract.module().storage().ident();
        let no_cross_calling_cfg =
            self.generate_code_using::<generator::CrossCallingConflictCfg>();
        let event_idents = self
            .contract
            .module()
            .events()
            .map(|event| event.ident())
            .collect::<Vec<_>>();
        let base_event_ident =
            proc_macro2::Ident::new("__ink_EventBase", Span::call_site());
        quote! {
            #no_cross_calling_cfg
            #[derive(::scale::Encode, ::scale::Decode)]
            pub enum #base_event_ident {
                #( #event_idents(#event_idents), )*
            }

            #no_cross_calling_cfg
            const _: () = {
                impl ::ink_lang::BaseEvent for #storage_ident {
                    type Type = #base_event_ident;
                }
            };

            #(
                #no_cross_calling_cfg
                const _: () = {
                    impl From<#event_idents> for #base_event_ident {
                        fn from(event: #event_idents) -> Self {
                            Self::#event_idents(event)
                        }
                    }
                };
            )*

            const _: () = {
                #no_cross_calling_cfg
                impl ::ink_core::env::Topics<EnvTypes> for #base_event_ident {
                    fn topics(&self) -> &'static [Hash] {
                        match self {
                            #(
                                Self::#event_idents(event) => {
                                    <#event_idents as ::ink_core::env::Topics<EnvTypes>>::topics(event),
                                }
                            )*
                        }
                    }
                }
            };
        }
    }

    /// Generates the `Topics` trait implementations for the user defined events.
    fn generate_topics_impls<'a>(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        let no_cross_calling_cfg =
            self.generate_code_using::<generator::CrossCallingConflictCfg>();
        self.contract.module().events().map(move |event| {
            let span = event.span();
            let ident = event.ident();

            quote_spanned!(span =>
                #no_cross_calling_cfg
                const _: () = {
                    impl ::ink_core::env::Topics<EnvTypes> for #ident {
                        fn topics(&self) -> &'static [Hash] {
                            // Issue: https://github.com/paritytech/ink/issues/105
                            &[]
                        }
                    }
                };
            )
        })
    }

    /// Generates all the user defined event struct definitions.
    fn generate_event_structs<'a>(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        let no_cross_calling_cfg =
            self.generate_code_using::<generator::CrossCallingConflictCfg>();
        self.contract.module().events().map(move |event| {
            let span = event.span();
            let ident = event.ident();
            let attrs = event.attrs();
            let fields = event.fields().map(|event_field| {
                let span = event_field.span();
                let attrs = event_field.attrs();
                let vis = event_field.vis();
                let ident = event_field.ident();
                let ty = event_field.ty();
                quote_spanned!(span=>
                    #( #attrs )*
                    #vis #ident : #ty
                )
            });
            quote_spanned!(span =>
                #no_cross_calling_cfg
                #( #attrs )*
                #[derive(scale::Encode, scale::Decode)]
                pub struct #ident {
                    #( #fields ),*
                }
            )
        })
    }
}
