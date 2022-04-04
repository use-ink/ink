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
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::{
    quote,
    quote_spanned,
};
use std::marker::PhantomData;
use syn::spanned::Spanned as _;

/// Generates code for an event definition.
#[derive(From)]
pub struct EventDefinition<'a> {
    event_def: &'a ir::InkEventDefinition,
}

impl GenerateCode for EventDefinition<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let emit_event_trait_impl = self.generate_emit_event_trait_impl();
        let topic_guards = self.generate_topic_guard();
        let topics_impls = self.generate_topics_impl();
        let event_structs = self.generate_event_struct();
        quote! {
            #emit_event_trait_impl
            #( #topic_guards )*
            #( #event_structs )*
            #( #topics_impls )*
        }
    }
}

impl<'a> EventDefinition<'a> {
    fn generate_event_struct(&'a self) -> TokenStream2 {
        let span = self.event_def.span();
        let ident = self.event_def.ident();
        let attrs = self.event_def.attrs();
        let fields = self.event_def.fields().map(|event_field| {
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
            #( #attrs )*
            #[derive(scale::Encode, scale::Decode)]
            pub struct #ident {
                #( #fields ),*
            }
        )
    }

    /// Generate checks to guard against too many topics in event definitions.
    fn generate_topics_guard(&self) -> TokenStream2 {
        let span = self.event_def.span();
        let event_ident = self.event_def.ident();
        let len_topics = self
            .event_def
            .fields()
            .filter(|event| event.is_topic)
            .count();
        quote_spanned!(span=>
            impl ::ink_lang::codegen::EventLenTopics for #event_ident {
                type LenTopics = ::ink_lang::codegen::EventTopics<#len_topics>;
            }
        )
    }

    /// Generates the `Topics` trait implementations for the user defined events.
    fn generate_topics_impls(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        let span = self.event_def.span();
        let event_ident = self.event_def.ident();
        let len_topics = self.event_def.fields().filter(|field| field.is_topic).count();
        let topic_impls = self.event_def
            .fields()
            .enumerate()
            .filter(|(_, field)| field.is_topic)
            .map(|(n, topic_field)| {
                let span = topic_field.span();
                let field_ident = topic_field
                    .ident()
                    .map(quote::ToTokens::into_token_stream)
                    .unwrap_or_else(|| quote_spanned!(span => #n));
                let field_type = topic_field.ty();
                quote_spanned!(span =>
                        .push_topic::<::ink_env::topics::PrefixedValue<#field_type>>(
                            &::ink_env::topics::PrefixedValue {
                                value: &self.#field_ident,
                                // todo: deduplicate with EVENT_SIGNATURE
                                prefix: ::core::concat!(
                                    ::core::module_path!(),
                                    "::",
                                    ::core::stringify!(#event_ident),
                                    "::",
                                    ::core::stringify!(#field_ident),
                                ).as_bytes(),
                            }
                        )
                    )
            });
        // Only include topic for event signature in case of non-anonymous event.
        let event_signature_topic = match self.event_def.anonymous {
            true => None,
            false => {
                Some(quote_spanned!(span=>
                    .push_topic::<::ink_env::topics::PrefixedValue<[u8; EVENT_SIGNATURE.len()]>>(
                        &::ink_env::topics::PrefixedValue {
                            value: EVENT_SIGNATURE, prefix: b""
                        }
                    )
                ))
            }
        };
        // Anonymous events require 1 fewer topics since they do not include their signature.
        let anonymous_topics_offset = if self.event_def.anonymous { 0 } else { 1 };
        let remaining_topics_ty = match len_topics + anonymous_topics_offset {
            0 => quote_spanned!(span=> ::ink_env::topics::state::NoRemainingTopics),
            n => {
                quote_spanned!(span=> [::ink_env::topics::state::HasRemainingTopics; #n])
            }
        };
        quote_spanned!(span =>
            const _: () = {
                impl ::ink_env::Topics for #event_ident {
                    type RemainingTopics = #remaining_topics_ty;

                    fn topics<E, B>(
                        &self,
                        builder: ::ink_env::topics::TopicsBuilder<::ink_env::topics::state::Uninit, E, B>,
                    ) -> <B as ::ink_env::topics::TopicsBuilderBackend<E>>::Output
                    where
                        E: ::ink_env::Environment,
                        B: ::ink_env::topics::TopicsBuilderBackend<E>,
                    {
                        const EVENT_SIGNATURE: &[u8] = <Self as ::ink_lang::reflect::ContractEvent>::PATH.as_bytes();

                        builder
                            .build::<Self>()
                            #event_signature_topic
                            #(
                                #topic_impls
                            )*
                            .finish()
                    }
                }
            };
        )
    }
}
