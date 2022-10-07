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
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

/// Generates code for an event definition.
#[derive(From)]
pub struct EventDefinition<'a> {
    event_def: &'a ir::InkEventDefinition,
}

impl GenerateCode for EventDefinition<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let event_enum = self.generate_event_enum();
        // let event_metadata_impl = self.generate_event_metadata_impl();
        let event_info_impls = self.generate_event_variant_info_impls();
        let topics_impl = self.generate_topics_impl();
        // let topics_guard = self.generate_topics_guard();
        quote! {
            #event_enum
            #event_info_impls
            // #event_metadata_impl
            #topics_impl
            // #topics_guard
        }
    }
}

impl<'a> EventDefinition<'a> {
    fn generate_event_enum(&'a self) -> TokenStream2 {
        let span = self.event_def.span();
        let event_enum = &self.event_def.item;
        quote_spanned!(span =>
            #[derive(::scale::Encode, ::scale::Decode)]
            #event_enum
        )
    }

    fn generate_event_variant_info_impls(&self) -> TokenStream2 {
        let span = self.event_def.span();
        let event_ident = self.event_def.ident();

        let impls = self.event_def.variants().map(|ev| {
            let event_variant_ident = ev.ident();
            let index = ev.index();
            quote_spanned!(span=>
                impl ::ink::reflect::EventVariantInfo<#index> for #event_ident {
                    const NAME: &'static str = ::core::stringify!(#event_ident);
                    // const SIGNATURE: [u8; 32] = ::ink::blake2x256!(::core::concat!(
                    //     ::core::module_path!(), "::",
                    //     ::core::stringify!(#event_ident), "::",
                    //     ::core::stringify!(#event_variant_ident))
                    // );
                    const SIGNATURE: [u8; 32] = ::ink::reflect::event_variant_signature(
                        ::core::module_path!(),
                        ::core::stringify!(#event_ident),
                        ::core::stringify!(#event_variant_ident),
                    );
                }
            )
        });
        quote_spanned!(span=>
            #(
                #impls
            )*
        )
    }

    fn generate_event_metadata_impl(&self) -> TokenStream2 {
        let event_metadata = super::metadata::EventMetadata::from(self.event_def);
        event_metadata.generate_code()
    }

    /// Generate checks to guard against too many topics in event definitions.
    fn generate_topics_guard(&self) -> TokenStream2 {
        let span = self.event_def.span();
        let event_ident = self.event_def.ident();
        // todo: [AJ] check if event signature topic should be included here (it is now, wasn't before)
        let len_topics = self.event_def.max_len_topics();

        quote_spanned!(span=>
            impl ::ink::codegen::EventLenTopics for #event_ident {
                type LenTopics = ::ink::codegen::EventTopics<#len_topics>;
            }
        )
    }

    fn generate_topics_impl(&self) -> TokenStream2 {
        let span = self.event_def.span();
        let event_ident = self.event_def.ident();
        let len_topics = self
            .event_def
            .max_len_topics();

        let variant_match_arms = self
            .event_def
            .variants()
            .map(|variant| {
                let span = variant.span();
                let variant_ident = variant.ident();
                let field_bindings = variant.fields()
                    .map(|field| {
                        let span = field.span();
                        let field_ident = field.ident();
                        quote_spanned!(span=> ref #field_ident)
                    });
                let field_topics = variant.fields()
                    .map(|field| {
                        let field_type = field.ty();
                        let field_ident = field.ident();
                        quote_spanned!(span =>
                            .push_topic::<::ink_env::topics::PrefixedValue<#field_type>>(
                                &::ink_env::topics::PrefixedValue {
                                    // todo: deduplicate with EVENT_SIGNATURE
                                    prefix: ::core::concat!(
                                        ::core::module_path!(),
                                        "::",
                                        ::core::stringify!(#event_ident),
                                        "::",
                                        ::core::stringify!(#field_ident),
                                    ).as_bytes(),
                                    value: &self.#field_ident,
                                }
                            )
                        )
                    });

                let event_signature_topic = match variant.anonymous() {
                    true => None,
                    false => {
                        Some(quote_spanned!(span=>
                            .push_topic::<::ink_env::topics::PrefixedValue<()>>(
                                &::ink_env::topics::PrefixedValue {
                                    // todo: look up event signature topic via indexed trait impl
                                    prefix: EVENT_SIGNATURE, value: &(),
                                }
                            )
                        ))
                    }
                };

                let remaining_topics_ty = match len_topics {
                    0 => quote_spanned!(span=> ::ink::env::topics::state::NoRemainingTopics),
                    n => {
                        quote_spanned!(span=> [::ink::env::topics::state::HasRemainingTopics; #n])
                    }
                };

                quote_spanned!(span=>
                    Self::#variant_ident { #( #field_bindings, )* } => {
                        builder
                            .build::<#remaining_topics_ty>()
                            #event_signature_topic
                            #(
                                #field_topics
                            )*
                            .finish()
                    }
                )
            });

        quote_spanned!(span =>
            const _: () = {
                impl ::ink_env::Topics for #event_ident {
                    fn topics<E, B>(
                        &self,
                        builder: ::ink::env::topics::TopicsBuilder<::ink::env::topics::state::Uninit, E, B>,
                    ) -> <B as ::ink::env::topics::TopicsBuilderBackend<E>>::Output
                    where
                        E: ::ink::env::Environment,
                        B: ::ink::env::topics::TopicsBuilderBackend<E>,
                    {
                        match self {
                            #(
                                #variant_match_arms
                            )*
                        }
                    }
                }
            };
        )
    }
}
