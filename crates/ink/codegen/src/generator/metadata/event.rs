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

use crate::GenerateCode;

use derive_more::From;
use ir::IsDocAttribute;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;

/// Generates code for an event definition.
#[derive(From)]
pub struct EventMetadata<'a> {
    event_def: &'a ir::InkEventDefinition,
}

impl GenerateCode for EventMetadata<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let span = self.event_def.span();
        let event_ident = self.event_def.ident();
        let docs = self
            .event_def
            .attrs()
            .iter()
            .filter_map(|attr| attr.extract_docs());
        let variants = self.event_def.variants().map(|ev| {
            let span = ev.span();
            let label = ev.ident();

            let args = ev.fields().map(|event_field| {
                let span = event_field.span();
                let ident = event_field.ident();
                let is_topic = event_field.is_topic;
                let docs = event_field
                    .attrs()
                    .into_iter()
                    .filter_map(|attr| attr.extract_docs());
                let ty = super::generate_type_spec(event_field.ty());
                quote_spanned!(span =>
                    ::ink::metadata::EventParamSpec::new(::core::stringify!(#ident))
                        .of_type(#ty)
                        .indexed(#is_topic)
                        .docs([
                            #( #docs ),*
                        ])
                        .done()
                )
            });

            let docs = ev
                .attrs()
                .iter()
                .filter_map(|attr| attr.extract_docs())
                .collect::<Vec<_>>();

            quote_spanned!(span=>
                ::ink::metadata::EventVariantSpec::new(::core::stringify!(#label))
                    .args([
                        #( #args ),*
                    ])
                    .docs([
                        #( #docs ),*
                    ])
                    .done()
            )
        });

        quote_spanned!(span=>
            /// This adds the unique id of the event definition into a custom section, which can
            /// be used by `cargo-contract` to identify and extract metadata for all imported event
            /// definitions in the Wasm binary.
            #[cfg(not(feature = "std"))]
            #[link_section = "__ink_event_definition_ids"]
            pub static __INK_EVENT_METADATA: u128 = ::ink::primitives::path_unique_id(
                <#event_ident as ::ink::reflect::EventInfo>::PATH
            );

            #[cfg(feature = "std")]
            #[cfg(not(feature = "ink-as-dependency"))]
            const _: () = {
                const EVENT_DEF_ID: u128 = ::ink::primitives::path_unique_id(
                    <#event_ident as ::ink::reflect::EventInfo>::PATH
                );

                impl ::ink::reflect::EventDefinition<{ EVENT_DEF_ID }> for ::ink::reflect::EventDefinitionRegistry {
                    type Type = #event_ident;
                }

                impl ::ink::metadata::EventMetadata for #event_ident {
                    fn event_spec() -> ::ink::metadata::EventSpec {
                        ::ink::metadata::EventSpec::new(<#event_ident as ::ink::reflect::EventInfo>::PATH)
                            .variants([
                                #( #variants ),*
                            ])
                            .docs([
                                #( #docs ),*
                            ])
                            .done()
                    }
                }
            };
        )
    }
}
