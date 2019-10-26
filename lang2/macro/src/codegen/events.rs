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
    codegen::GenerateCode,
    ir,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};

/// Generates code to generate the metadata of the contract.
#[derive(From)]
pub struct Events<'a> {
    /// The contract to generate code for.
    contract: &'a ir::Contract,
}

impl GenerateCode for Events<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let event_structs = self.generate_event_structs();
        let event_enum = self.generate_event_enum();
        let emit_event_trait = self.generate_emit_event_trait();
        let exports = self.generate_exports();

        // Generate no code if there are no user defined events.
        if self.contract.events.is_empty() {
            return quote! {}
        }

        quote! {
            mod __ink_events {
                use super::*;

                #(#event_structs)*

                pub mod __ink_private {
                    use super::*;

                    #event_enum
                    #emit_event_trait
                }
            }

            #exports
        }
    }
}

impl Events<'_> {
    fn generate_exports(&self) -> TokenStream2 {
        let event_idents = self
            .contract
            .events
            .iter()
            .map(|item_event| &item_event.ident);

        quote! {
            pub use __ink_events::{
                #( #event_idents ),*
            };
        }
    }

    fn generate_event_enum(&self) -> TokenStream2 {
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

    fn generate_emit_event_trait(&self) -> TokenStream2 {
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

    fn generate_event_structs<'a>(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract.events.iter().map(|item_event| {
            let span = item_event.span();
            let ident = &item_event.ident;
            use core::convert::TryFrom as _;
            let attrs = item_event
                .attrs
                .iter()
                .filter(|&attr| ir::Marker::try_from(attr.clone()).is_err());
            let mut fields = item_event.fields.clone();
            fields
                .named
                .iter_mut()
                .for_each(|field| {
                    // Set visibility of all fields to `pub`.
                    field.vis = syn::Visibility::Public(syn::VisPublic {
                        pub_token: Default::default(),
                    });
                    // Only re-generate non-ink! attributes.
                    field.attrs.retain(|attr| ir::Marker::try_from(attr.clone()).is_err())
                });

            quote_spanned!(span =>
                #(#attrs)*
                #[derive(scale::Encode)]
                pub struct #ident
                    #fields

                impl ink_core::env2::Topics<Env> for #ident {
                    fn topics(&self) -> &'static [Hash] {
                        &[]
                    }
                }
            )
        })
    }
}
