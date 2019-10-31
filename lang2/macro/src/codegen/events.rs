// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    codegen::{
        cross_calling::CrossCallingConflictCfg,
        env_types::EnvTypesImports,
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
        let conflic_depedency_cfg = self.generate_code_using::<CrossCallingConflictCfg>();
        let topics_impls = self.generate_topics_impls();
        let event_enum = self.generate_event_enum();
        let emit_event_trait = self.generate_emit_event_trait();
        let event_imports = self.generate_code_using::<EventImports>();
        let env_imports = self.generate_code_using::<EnvTypesImports>();

        // Generate no code if there are no user defined events.
        if self.contract.events.is_empty() {
            return quote! {}
        }

        quote! {
            #conflic_depedency_cfg
            mod __ink_events {
                #env_imports
                #event_imports

                #(
                    #topics_impls
                )*

                #event_enum
                #emit_event_trait
            }
            #conflic_depedency_cfg
            pub use __ink_events::{EmitEvent, Event};
        }
    }
}

impl EventHelpers<'_> {
    fn generate_emit_event_trait(&self) -> TokenStream2 {
        quote! {
            pub trait EmitEvent {
                fn emit_event<E>(self, event: E)
                where
                    E: Into<Event>;
            }

            impl<'a> EmitEvent for &'a mut ink_core::env2::EnvAccessMut<Env> {
                fn emit_event<E>(self, event: E)
                where
                    E: Into<Event>,
                {
                    ink_core::env2::EmitEvent::emit_event(self, event.into())
                }
            }
        }
    }

    fn generate_event_enum(&self) -> TokenStream2 {
        let event_idents = self
            .contract
            .events
            .iter()
            .map(|item_event| &item_event.ident)
            .collect::<Vec<_>>();

        quote! {
            #[derive(scale::Encode)]
            pub enum Event {
                #( #event_idents(#event_idents), )*
            }

            #(
                impl From<#event_idents> for Event {
                    fn from(event: #event_idents) -> Self {
                        Event::#event_idents(event)
                    }
                }
            )*

            impl ink_core::env2::Topics<Env> for Event {
                fn topics(&self) -> &'static [Hash] {
                    match self {
                        #(
                            Event::#event_idents(event) => event.topics(),
                        )*
                    }
                }
            }
        }
    }

    fn generate_topics_impls<'a>(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract.events.iter().map(|item_event| {
            let span = item_event.span();
            let ident = &item_event.ident;

            quote_spanned!(span =>
                impl ink_core::env2::Topics<Env> for #ident {
                    fn topics(&self) -> &'static [Hash] {
                        &[]
                    }
                }
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
/// - add `#[derive(scale::Encode)]`
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
        self.contract.events.iter().map(move |item_event| {
            let conflic_depedency_cfg =
                self.generate_code_using::<CrossCallingConflictCfg>();

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
                #conflic_depedency_cfg
                #(#attrs)*
                #[derive(scale::Encode)]
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

/// Generates code to generate the event imports mainly used by
/// definitions inside of the generated `__ink_private` module.
///
/// # Note
///
/// The generated code can be used from arbitrary positions within
/// the `__ink_private` module.
#[derive(From)]
pub struct EventImports<'a> {
    /// The contract to generate code for.
    contract: &'a ir::Contract,
}

impl<'a> GenerateCodeUsing for EventImports<'a> {
    fn contract(&self) -> &ir::Contract {
        self.contract
    }
}

impl GenerateCode for EventImports<'_> {
    fn generate_code(&self) -> TokenStream2 {
        if self.contract.events.is_empty() {
            return quote! {}
        }

        let conflic_depedency_cfg = self.generate_code_using::<CrossCallingConflictCfg>();
        let event_idents = self
            .contract
            .events
            .iter()
            .map(|item_event| &item_event.ident);

        quote! {
            #conflic_depedency_cfg
            pub use super::{
                #( #event_idents ),*
            };
        }
    }
}
