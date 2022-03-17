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
use syn::spanned::Spanned as _;

/// Generates code for the ink! event structs of the contract.
#[derive(From)]
pub struct Events<'a> {
    contract: &'a ir::Contract,
}
impl_as_ref_for_generator!(Events);

impl GenerateCode for Events<'_> {
    fn generate_code(&self) -> TokenStream2 {
        if self.contract.module().events().next().is_none() {
            // Generate no code in case there are no event definitions.
            return TokenStream2::new()
        }
        let emit_event_trait_impl = self.generate_emit_event_trait_impl();
        let event_base = self.generate_event_base();
        let topic_guards = self.generate_topic_guards();
        let topics_impls = self.generate_topics_impls();
        let event_structs = self.generate_event_structs();
        quote! {
            #emit_event_trait_impl
            #event_base
            #( #topic_guards )*
            #( #event_structs )*
            #( #topics_impls )*
        }
    }
}

impl<'a> Events<'a> {
    /// Used to allow emitting user defined events directly instead of converting
    /// them first into the automatically generated base trait of the contract.
    fn generate_emit_event_trait_impl(&self) -> TokenStream2 {
        let storage_ident = &self.contract.module().storage().ident();
        quote! {
            const _: () = {
                impl<'a> ::ink_lang::codegen::EmitEvent<#storage_ident> for ::ink_lang::EnvAccess<'a, Environment> {
                    fn emit_event<E>(self, event: E)
                    where
                        E: Into<<#storage_ident as ::ink_lang::reflect::ContractEventBase>::Type>,
                    {
                        ::ink_env::emit_event::<
                            Environment,
                            <#storage_ident as ::ink_lang::reflect::ContractEventBase>::Type
                        >(event.into());
                    }
                }
            };
        }
    }

    /// Generates the base event enum that comprises all user defined events.
    /// All emitted events are converted into a variant of this enum before being
    /// serialized and emitted to apply their unique event discriminant (ID).
    ///
    /// # Developer Note
    ///
    /// The `__ink_dylint_EventBase` config attribute is used here to convey the
    /// information that the generated enum is an ink! event to `dylint`.
    fn generate_event_base(&self) -> TokenStream2 {
        let storage_ident = &self.contract.module().storage().ident();
        let event_idents = self
            .contract
            .module()
            .events()
            .map(|event| event.ident())
            .collect::<Vec<_>>();
        let base_event_ident =
            proc_macro2::Ident::new("__ink_EventBase", Span::call_site());
        quote! {
            #[allow(non_camel_case_types)]
            #[derive(::scale::Encode, ::scale::Decode)]
            #[cfg(not(feature = "__ink_dylint_EventBase"))]
            pub enum #base_event_ident {
                #( #event_idents(#event_idents), )*
            }

            const _: () = {
                impl ::ink_lang::reflect::ContractEventBase for #storage_ident {
                    type Type = #base_event_ident;
                }
            };

            #(
                const _: () = {
                    impl From<#event_idents> for #base_event_ident {
                        fn from(event: #event_idents) -> Self {
                            Self::#event_idents(event)
                        }
                    }
                };
            )*

            const _: () = {
                pub enum __ink_UndefinedAmountOfTopics {}
                impl ::ink_env::topics::EventTopicsAmount for __ink_UndefinedAmountOfTopics {
                    const AMOUNT: usize = 0;
                }

                impl ::ink_env::Topics for #base_event_ident {
                    type RemainingTopics = __ink_UndefinedAmountOfTopics;

                    fn topics<E, B>(
                        &self,
                        builder: ::ink_env::topics::TopicsBuilder<::ink_env::topics::state::Uninit, E, B>,
                    ) -> <B as ::ink_env::topics::TopicsBuilderBackend<E>>::Output
                    where
                        E: ::ink_env::Environment,
                        B: ::ink_env::topics::TopicsBuilderBackend<E>,
                    {
                        match self {
                            #(
                                Self::#event_idents(event) => {
                                    <#event_idents as ::ink_env::Topics>::topics::<E, B>(event, builder)
                                }
                            )*
                        }
                    }
                }
            };
        }
    }

    /// Generate checks to guard against too many topics in event definitions.
    fn generate_topics_guard(&self, event: &ir::Event) -> TokenStream2 {
        let span = event.span();
        let storage_ident = self.contract.module().storage().ident();
        let event_ident = event.ident();
        let len_topics = event.fields().filter(|event| event.is_topic).count();
        let max_len_topics = quote_spanned!(span=>
            <<#storage_ident as ::ink_lang::reflect::ContractEnv>::Env
                as ::ink_env::Environment>::MAX_EVENT_TOPICS
        );
        quote_spanned!(span=>
            impl ::ink_lang::codegen::EventLenTopics for #event_ident {
                type LenTopics = ::ink_lang::codegen::EventTopics<#len_topics>;
            }

            const _: () = ::ink_lang::codegen::utils::consume_type::<
                ::ink_lang::codegen::EventRespectsTopicLimit<
                    #event_ident,
                    { #max_len_topics },
                >
            >();
        )
    }

    /// Generates the guard code that protects against having too many topics defined on an ink! event.
    fn generate_topic_guards(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract.module().events().map(move |event| {
            let span = event.span();
            let topics_guard = self.generate_topics_guard(event);
            quote_spanned!(span =>
                #topics_guard
            )
        })
    }

    /// Generates the `Topics` trait implementations for the user defined events.
    fn generate_topics_impls(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        let contract_ident = self.contract.module().storage().ident();
        self.contract.module().events().map(move |event| {
            let span = event.span();
            let event_ident = event.ident();
            let event_signature = syn::LitByteStr::new(
                format!("{}::{}", contract_ident, event_ident
            ).as_bytes(), span);
            let len_event_signature = event_signature.value().len();
            let len_topics = event.fields().filter(|field| field.is_topic).count();
            let topic_impls = event
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
                    let signature = syn::LitByteStr::new(
                        format!("{}::{}::{}", contract_ident, event_ident,
                            field_ident
                        ).as_bytes(), span);
                    quote_spanned!(span =>
                        .push_topic::<::ink_env::topics::PrefixedValue<#field_type>>(
                            &::ink_env::topics::PrefixedValue { value: &self.#field_ident, prefix: #signature }
                        )
                    )
                });
            // Only include topic for event signature in case of non-anonymous event.
            let event_signature_topic = match event.anonymous {
                true => None,
                false => Some(quote_spanned!(span=>
                    .push_topic::<::ink_env::topics::PrefixedValue<[u8; #len_event_signature]>>(
                        &::ink_env::topics::PrefixedValue { value: #event_signature, prefix: b"" }
                    )
                ))
            };
            // Anonymous events require 1 fewer topics since they do not include their signature.
            let anonymous_topics_offset = if event.anonymous { 0 } else { 1 };
            let remaining_topics_ty = match len_topics + anonymous_topics_offset {
                0 => quote_spanned!(span=> ::ink_env::topics::state::NoRemainingTopics),
                n => quote_spanned!(span=> [::ink_env::topics::state::HasRemainingTopics; #n]),
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
        })
    }

    /// Generates all the user defined event struct definitions.
    fn generate_event_structs(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
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
                #( #attrs )*
                #[derive(scale::Encode, scale::Decode)]
                pub struct #ident {
                    #( #fields ),*
                }
            )
        })
    }
}
