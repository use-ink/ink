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

mod metadata;

pub use metadata::event_metadata_derive;

use ink_codegen::generate_code;
use ink_ir::EventConfig;
use ink_primitives::abi::Abi;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::{
    Token,
    punctuated::Punctuated,
    spanned::Spanned,
};

/// Generate code from the `#[ink::event]` attribute. This expands to the required
/// derive macros to satisfy an event implementation.
pub fn generate(config: TokenStream2, input: TokenStream2) -> TokenStream2 {
    ink_ir::Event::new(config, input)
        .map(|event| generate_code(&event))
        .unwrap_or_else(|err| err.to_compile_error())
}

/// Derives the `ink::Event` trait for the given `struct`.
pub fn event_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::Move)
        .add_bounds(synstructure::AddBounds::Fields)
        .underscore_const(true);
    match &s.ast().data {
        syn::Data::Struct(_) => {
            event_derive_struct(s).unwrap_or_else(|err| err.to_compile_error())
        }
        _ => {
            syn::Error::new(
                s.ast().span(),
                "can only derive `Event` for Rust `struct` items",
            )
            .to_compile_error()
        }
    }
}

/// `Event` derive implementation for `struct` types.
#[allow(clippy::arithmetic_side_effects)] // todo
fn event_derive_struct(s: synstructure::Structure) -> syn::Result<TokenStream2> {
    assert_eq!(s.variants().len(), 1, "can only operate on structs");

    if !s.ast().generics.params.is_empty() {
        return Err(syn::Error::new(
            s.ast().generics.params.span(),
            "can only derive `Event` for structs without generics",
        ));
    }

    let span = s.ast().span();
    let config = EventConfig::try_from(s.ast().attrs.as_slice())?;
    let anonymous = config.anonymous();
    let variant = &s.variants()[0];

    // Partition field bindings between topic and data fields.
    let mut topic_fields = Vec::new();
    let mut data_fields = Vec::new();
    let mut topic_err: Option<syn::Error> = None;
    for field in variant.bindings() {
        match has_ink_topic_attribute(field) {
            Ok(is_topic) => {
                if is_topic {
                    topic_fields.push(field);
                } else {
                    data_fields.push(field);
                }
            }
            Err(err) => {
                match topic_err {
                    Some(ref mut topic_err) => topic_err.combine(err),
                    None => topic_err = Some(err),
                }
            }
        }
    }
    if let Some(err) = topic_err {
        return Err(err);
    }

    // Anonymous events require 1 fewer topics since they do not include their signature.
    let anonymous_topics_offset = usize::from(!anonymous);
    let len_topics = topic_fields.len() + anonymous_topics_offset;

    // Enforces `pallet-revive` and Solidity ABI topic limits.
    // Ref: <https://github.com/paritytech/polkadot-sdk/blob/7ede4fd048f8a99e62ef31050aa2e167e99d54b9/substrate/frame/revive/src/limits.rs#L46-L49>
    // Ref: <https://docs.soliditylang.org/en/latest/abi-spec.html#events>
    if len_topics > 4 {
        return Err(syn::Error::new(
            span,
            format!(
                "Events{} can only have up to {} fields annotated with an \
                `#[ink(topic)]` attribute",
                if anonymous {
                    " with an `anonymous` attribute argument"
                } else {
                    ""
                },
                if anonymous { 4 } else { 3 }
            ),
        ));
    }

    let remaining_topics_ty = match len_topics {
        0 => quote_spanned!(span=> ::ink::env::event::state::NoRemainingTopics),
        _ => {
            quote_spanned!(span=> [::ink::env::event::state::HasRemainingTopics; #len_topics])
        }
    };

    Ok(generate_abi_impls!(@type |abi| {
        let abi_ty = match abi {
            Abi::Ink => quote!(::ink::abi::Ink),
            Abi::Sol => quote!(::ink::abi::Sol),
        };

        let event_signature_topic = if anonymous {
            None
        } else {
            let value = match abi {
                Abi::Ink => quote!(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC.as_ref()),
                Abi::Sol => {
                    quote! {
                        ::ink::sol::FixedBytes::from_ref(
                            <Self as ::ink::env::Event<::ink::abi::Sol>>::SIGNATURE_TOPIC
                                .as_ref()
                                .expect("Expected a signature topic")
                        )
                    }
                }
            };
            Some(quote_spanned!(span=>
                .push_topic(#value)
            ))
        };

        let signature_topic = if !anonymous {
            let event_name = config
                .name()
                .map(ToString::to_string)
                .unwrap_or_else(|| variant.ast().ident.to_string());
            match abi {
                Abi::Ink => {
                    if let Some(sig_arg) = config.signature_topic() {
                        let bytes = sig_arg.to_bytes();
                        quote_spanned!(span=> ::core::option::Option::Some([ #(#bytes),* ]))
                    } else {
                        let calculated_signature_topic =
                            signature_topic(variant.ast().fields, event_name);
                        quote_spanned!(span=> ::core::option::Option::Some(#calculated_signature_topic))
                    }
                }
                Abi::Sol => {
                    let calculated_signature_topic =
                        signature_topic_sol(variant.ast().fields, event_name);
                    quote_spanned!(span=> ::core::option::Option::Some(#calculated_signature_topic))
                }
            }
        } else {
            quote_spanned!(span=> ::core::option::Option::None)
        };

        let topics = topic_fields.iter().fold(quote!(), |acc, field| {
            let field_ty = &field.ast().ty;
            let field_span = field_ty.span();
            let value = match abi {
                Abi::Ink => quote!(::ink::as_option!(#field)),
                Abi::Sol => quote!(#field),
            };
            quote_spanned!(field_span=>
                #acc
                .push_topic(#value)
            )
        });
        let pat = variant.pat();
        let topics_builder = quote!(
            #pat => {
                builder
                    .build::<Self>()
                    #event_signature_topic
                    #topics
                    .finish()
            }
        );

        let encode_data = match abi {
            Abi::Ink => quote! {
                ::ink::abi::AbiEncodeWith::<::ink::abi::Ink>::encode_with(self)
            },
            Abi::Sol => {
                // For Solidity ABI encoding, only un-indexed fields are encoded as data.
                let data_field_tys = data_fields.iter().map(|field| {
                    let ty = &field.ast().ty;
                    quote!( &#ty )
                });
                let data_field_values = data_fields.iter().map(|field| {
                    &field.binding
                });
                quote! {
                    match self {
                        #pat => {
                            ::ink::sol::encode_sequence::<( #( #data_field_tys, )* )>(
                                &( #( #data_field_values, )* ),
                            )
                        }
                    }
                }
            },
        };

        s.bound_impl(quote!(::ink::env::Event<#abi_ty>), quote! {
            type RemainingTopics = #remaining_topics_ty;
            const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> = #signature_topic;

            fn topics<B>(
                &self,
                builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, B, #abi_ty>,
            ) -> <B as ::ink::env::event::TopicsBuilderBackend<#abi_ty>>::Output
            where
                B: ::ink::env::event::TopicsBuilderBackend<#abi_ty>,
            {
                match self {
                    #topics_builder
                }
            }

            fn encode_data(&self) -> ::ink::prelude::vec::Vec<::core::primitive::u8> {
                #encode_data
            }
        })
    }))
}

/// Checks if the given field's attributes contain an `#[ink(topic)]` attribute.
///
/// Returns `Err` if:
/// - the given attributes contain a `#[cfg(...)]` attribute
/// - there are `ink` attributes other than a single `#[ink(topic)]`
fn has_ink_topic_attribute(field: &synstructure::BindingInfo) -> syn::Result<bool> {
    let some_cfg_attrs = field
        .ast()
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("cfg"));
    if some_cfg_attrs.is_some() {
        Err(syn::Error::new(
            field.ast().span(),
            "conditional compilation is not allowed for event fields",
        ))
    } else {
        let attrs = parse_arg_attrs(&field.ast().attrs)?;
        has_ink_attribute(&attrs, "topic")
    }
}

/// Checks if the given attributes contain an `ink` attribute with the given path.
fn has_ink_attribute(ink_attrs: &[syn::Meta], path: &str) -> syn::Result<bool> {
    let mut present = false;
    for a in ink_attrs {
        if a.path().is_ident(path) && !present {
            present = true;
        } else if a.path().is_ident(path) {
            return Err(syn::Error::new(
                a.span(),
                format!("Only a single `#[ink({path})]` is allowed"),
            ));
        } else {
            return Err(syn::Error::new(
                a.span(),
                "Unknown ink! attribute at this position",
            ));
        }
    }
    Ok(present)
}

/// Parses custom `ink` attributes with the arbitrary arguments.
///
/// # Errors
/// - Attribute has no argument (i.e. `#[ink()]`)
fn parse_arg_attrs(attrs: &[syn::Attribute]) -> syn::Result<Vec<syn::Meta>> {
    let mut ink_attrs = Vec::new();
    for a in attrs {
        if !a.path().is_ident("ink") {
            continue;
        }

        let nested = a.parse_args_with(
            Punctuated::<syn::Meta, Token![,]>::parse_separated_nonempty,
        )?;
        if nested.is_empty() {
            return Err(syn::Error::new(
                a.span(),
                "Expected to have an argument".to_string(),
            ));
        }
        ink_attrs.extend(nested.into_iter())
    }

    Ok(ink_attrs)
}

/// The signature topic of an event variant.
///
/// Calculated with `blake2b("Event(field1_type,field2_type)")`.
fn signature_topic(fields: &syn::Fields, event_name: String) -> TokenStream2 {
    let fields = fields
        .iter()
        .map(|field| {
            quote::ToTokens::to_token_stream(&field.ty)
                .to_string()
                .replace(' ', "")
        })
        .collect::<Vec<_>>()
        .join(",");
    let topic_str = format!("{event_name}({fields})");
    quote!(::ink::blake2x256!(#topic_str))
}

/// The Solidity ABI signature topic of an event.
///
/// (i.e. the Keccak-256 hash of the Solidity ABI event signature).
fn signature_topic_sol(fields: &syn::Fields, event_name: String) -> TokenStream2 {
    let param_tys = fields.iter().map(|field| {
        let ty = &field.ty;
        quote! {
            <#ty as ::ink::SolEncode>::SOL_NAME
        }
    });
    let sig_arg_fmt_params = (0..fields.len())
        .map(|_| "{}")
        .collect::<Vec<_>>()
        .join(",");
    let sig_fmt_str = format!("{{}}({sig_arg_fmt_params})");
    let sig_str = quote! {
        ::ink::codegen::utils::const_format!(#sig_fmt_str, #event_name #(,#param_tys)*)
    };
    quote!(::ink::keccak_256!(#sig_str))
}
