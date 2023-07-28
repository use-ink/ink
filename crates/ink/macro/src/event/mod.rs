// Copyright (C) Parity Technologies (UK) Ltd.
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
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned;

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
fn event_derive_struct(mut s: synstructure::Structure) -> syn::Result<TokenStream2> {
    assert_eq!(s.variants().len(), 1, "can only operate on structs");

    if !s.ast().generics.params.is_empty() {
        return Err(syn::Error::new(
            s.ast().generics.params.span(),
            "can only derive `Event` for structs without generics",
        ))
    }

    let span = s.ast().span();
    let anonymous = has_ink_attribute(&s.ast().attrs, "anonymous")?;

    // filter field bindings to those marked as topics
    let mut topic_err: Option<syn::Error> = None;
    s.variants_mut()[0].filter(|bi| {
        match has_ink_topic_attribute(bi) {
            Ok(has_attr) => has_attr,
            Err(err) => {
                match topic_err {
                    Some(ref mut topic_err) => topic_err.combine(err),
                    None => topic_err = Some(err),
                }
                false
            }
        }
    });
    if let Some(err) = topic_err {
        return Err(err)
    }

    let variant = &s.variants()[0];

    // Anonymous events require 1 fewer topics since they do not include their signature.
    let anonymous_topics_offset = usize::from(!anonymous);
    let len_topics = variant.bindings().len() + anonymous_topics_offset;

    let remaining_topics_ty = match len_topics {
        0 => quote_spanned!(span=> ::ink::env::event::state::NoRemainingTopics),
        _ => {
            quote_spanned!(span=> [::ink::env::event::state::HasRemainingTopics; #len_topics])
        }
    };

    let event_ident = variant.ast().ident;
    let signature_topic = if !anonymous {
        let signature_topic = signature_topic(variant.ast().fields, event_ident);
        quote_spanned!(span=> ::core::option::Option::Some(#signature_topic))
    } else {
        quote_spanned!(span=> ::core::option::Option::None)
    };
    let event_signature_topic = if anonymous {
        None
    } else {
        Some(quote_spanned!(span=>
            .push_topic(Self::SIGNATURE_TOPIC.as_ref())
        ))
    };

    let topics = variant.bindings().iter().fold(quote!(), |acc, field| {
        let field_ty = &field.ast().ty;
        let field_span = field_ty.span();
        quote_spanned!(field_span=>
            #acc
            .push_topic(::ink::as_option!(#field))
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

    Ok(s.bound_impl(quote!(::ink::env::Event), quote! {
        type RemainingTopics = #remaining_topics_ty;

        const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> = #signature_topic;

        fn topics<E, B>(
            &self,
            builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, E, B>,
        ) -> <B as ::ink::env::event::TopicsBuilderBackend<E>>::Output
        where
            E: ::ink::env::Environment,
            B: ::ink::env::event::TopicsBuilderBackend<E>,
        {
            match self {
                #topics_builder
            }
        }
     }))
}

/// The signature topic of an event variant.
///
/// Calculated with `blake2b("Event(field1_type,field2_type)")`.
fn signature_topic(fields: &syn::Fields, event_ident: &syn::Ident) -> TokenStream2 {
    let fields = fields
        .iter()
        .map(|field| {
            quote::ToTokens::to_token_stream(&field.ty)
                .to_string()
                .replace(' ', "")
        })
        .collect::<Vec<_>>()
        .join(",");
    let topic_str = format!("{}({fields})", event_ident);
    quote!(::ink::blake2x256!(#topic_str))
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
        has_ink_attribute(&field.ast().attrs, "topic")
    }
}

/// Checks if the given attributes contain an `ink` attribute with the given path.
///
/// # Errors
/// - If there are multiple `ink` attributes with the given path.
/// - If multiple arguments are given to the `ink` attribute.
/// - If any other `ink` attributes are present other than the one with the given path.
fn has_ink_attribute(attrs: &[syn::Attribute], path: &str) -> syn::Result<bool> {
    let ink_attrs = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("ink") {
                let parse_result = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident(path) {
                        if meta.input.is_empty() {
                            Ok(())
                        } else {
                            Err(meta.error(format!(
                                "Invalid `#[ink({path})]` attribute: multiple arguments not allowed.",
                            )))
                        }
                    } else {
                        Err(meta
                            .error(format!("Only `#[ink({path})]` attribute allowed.")))
                    }
                });
                Some(parse_result.map(|_| attr))
            } else {
                None
            }
        })
        .collect::<syn::Result<Vec<_>>>()?;
    if ink_attrs.len() > 1 {
        return Err(syn::Error::new(
            ink_attrs[1].span(),
            format!("Only a single `#[ink({})]` attribute allowed.", path),
        ))
    }
    Ok(!ink_attrs.is_empty())
}
