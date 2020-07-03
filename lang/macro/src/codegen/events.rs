// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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
    codegen::{
        cross_calling::CrossCallingConflictCfg,
        GenerateCode,
        GenerateCodeUsing,
    },
    ir,
    ir::utils,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};

/// Generates helper definitions for the user defined event definitions.
///
/// These include:
///
/// - `Event` enum that unifies all user defined event definitions
/// - `EmitEvent` helper trait to allow for `emit_event` in messages and constructors
/// - `Topics` implementations for all user provided event definitions
///
/// # Note
///
/// All of this code should be generated inside the `__ink_private` module.
#[derive(From)]
pub struct EventHelpers<'a> {
    /// The contract to generate code for.
    contract: &'a ir::Contract,
}

impl<'a> GenerateCodeUsing for EventHelpers<'a> {
    fn contract(&self) -> &ir::Contract {
        self.contract
    }
}

impl GenerateCode for EventHelpers<'_> {
    fn generate_code(&self) -> TokenStream2 {
        if self.contract.events.is_empty() {
            return quote! {}
        }
        let topics_impls = self.generate_topics_impls();
        let event_enum = self.generate_event_enum();
        let emit_event_trait = self.generate_emit_event_trait();
        // Generate no code if there are no user defined events.
        quote! {
            #( #topics_impls )*
            #event_enum
            #emit_event_trait
        }
    }
}

impl EventHelpers<'_> {
    fn generate_emit_event_trait(&self) -> TokenStream2 {
        let storage_ident = &self.contract.storage.ident;
        quote! {
            const _: () = {
                impl<'a> ::ink_lang::EmitEvent<#storage_ident> for ::ink_lang::EnvAccess<'a, EnvTypes> {
                    fn emit_event<E>(self, event: E)
                    where
                        E: Into<<#storage_ident as ::ink_lang::BaseEvent>::Type>,
                    {
                        ::ink_core::env::emit_event::<
                            EnvTypes,
                            <#storage_ident as ::ink_lang::BaseEvent>::Type,
                        >(event.into());
                    }
                }
            };
        }
    }

    fn generate_event_enum(&self) -> TokenStream2 {
        let storage_ident = &self.contract.storage.ident;
        let event_idents = self
            .contract
            .events
            .iter()
            .map(|item_event| &item_event.ident)
            .collect::<Vec<_>>();
        let cfg = self.generate_code_using::<CrossCallingConflictCfg>();

        quote! {
            #cfg
            #[derive(scale::Encode, scale::Decode)]
            pub enum Event {
                #( #event_idents(#event_idents), )*
            }

            #cfg
            const _: () = {
                impl ::ink_lang::BaseEvent for #storage_ident {
                    type Type = Event;
                }
            };

            #(
                #cfg
                const _: () = {
                    impl From<#event_idents> for Event {
                        fn from(event: #event_idents) -> Self {
                            Event::#event_idents(event)
                        }
                    }
                };
            )*

            const _: () = {
                #cfg
                impl ::ink_core::env::Topics<EnvTypes> for Event {
                    fn topics(&self) -> &'static [Hash] {
                        match self {
                            #(
                                Event::#event_idents(event) => event.topics(),
                            )*
                        }
                    }
                }
            };
        }
    }

    fn generate_topics_impls<'a>(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        let cfg = self.generate_code_using::<CrossCallingConflictCfg>();
        self.contract.events.iter().map(move |item_event| {
            let span = item_event.span();
            let ident = &item_event.ident;

            quote_spanned!(span =>
                #cfg
                const _: () = {
                    impl ::ink_core::env::Topics<EnvTypes> for #ident {
                        fn topics(&self) -> &'static [Hash] {
                            &[]
                        }
                    }
                };
            )
        })
    }
}

/// Generates the user provided event `struct` definitions.
///
/// This includes
///
/// - making all fields `pub`
/// - strip `#[ink(..)]` attributes
/// - add `#[derive(scale::Encode, scale::Decode)]`
///
/// # Note
///
/// The code shall be generated on the ink! module root.
#[derive(From)]
pub struct EventStructs<'a> {
    /// The contract to generate code for.
    contract: &'a ir::Contract,
}

impl<'a> GenerateCodeUsing for EventStructs<'a> {
    fn contract(&self) -> &ir::Contract {
        self.contract
    }
}

impl EventStructs<'_> {
    fn generate_event_structs<'a>(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        let cfg = self.generate_code_using::<CrossCallingConflictCfg>();
        self.contract.events.iter().map(move |item_event| {
            let span = item_event.span();
            let ident = &item_event.ident;
            let attrs = utils::filter_non_ink_attributes(&item_event.attrs);
            let mut fields = item_event.fields.clone();
            fields.named.iter_mut().for_each(|field| {
                // Set visibility of all fields to `pub`.
                field.vis = syn::Visibility::Public(syn::VisPublic {
                    pub_token: Default::default(),
                });
                // Only re-generate non-ink! attributes.
                field
                    .attrs
                    .retain(|attr| !ir::utils::is_ink_attribute(attr))
            });

            quote_spanned!(span =>
                #cfg
                #(#attrs)*
                #[derive(scale::Encode, scale::Decode)]
                pub struct #ident
                    #fields
            )
        })
    }
}

impl GenerateCode for EventStructs<'_> {
    fn generate_code(&self) -> TokenStream2 {
        // Generate no code if there are no user defined events.
        if self.contract.events.is_empty() {
            return quote! {}
        }

        let event_structs = self.generate_event_structs();
        quote! {
            #(#event_structs)*
        }
    }
}
