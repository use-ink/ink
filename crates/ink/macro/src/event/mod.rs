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

use ink_ir::{
    format_err_spanned,
    utils::duplicate_config_err,
    SignatureTopicArg,
};
pub use metadata::event_metadata_derive;

use ink_codegen::generate_code;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    Token,
};

/// Event item configurations specified by nested `ink` attributes.
struct EventConfig {
    /// Event is anonymous.
    pub anonymous: bool,
    /// Event has a specified signature topic.
    pub signature_topic: Option<SignatureTopicArg>,
}

impl EventConfig {
    pub fn new(anonymous: bool, signature_topic: Option<SignatureTopicArg>) -> Self {
        EventConfig {
            anonymous,
            signature_topic,
        }
    }
}

impl TryFrom<&[syn::Meta]> for EventConfig {
    type Error = syn::Error;

    fn try_from(args: &[syn::Meta]) -> Result<Self, Self::Error> {
        let mut anonymous: Option<&syn::Meta> = None;
        let mut signature_topic: Option<&syn::Meta> = None;
        for arg in args.iter() {
            if arg.path().is_ident("anonymous") {
                if let Some(a_meta) = anonymous {
                    return Err(duplicate_config_err(a_meta, arg, "anonymous", "event"));
                }
                match arg {
                    syn::Meta::Path(_) => anonymous = Some(arg),
                    _ => {
                        return Err(format_err_spanned!(
                            arg,
                            "`#[ink(anonymous)]` takes no arguments",
                        ));
                    }
                }
            } else if arg.path().is_ident("signature_topic") {
                if anonymous.is_some() {
                    return Err(format_err_spanned!(
                        arg,
                        "cannot specify `signature_topic` with `anonymous` in ink! event item configuration argument",
                    ));
                }

                if let Some(lit_str) = signature_topic {
                    return Err(duplicate_config_err(lit_str, arg, "anonymous", "event"));
                }
                match arg {
                    syn::Meta::NameValue(_) => signature_topic = Some(arg),
                    _ => {
                        return Err(format_err_spanned!(
                            arg,
                            "expected a name-value pair",
                        ));
                    }
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! event item configuration argument",
                ));
            }
        }

        let signature_topic = if let Some(meta) = signature_topic {
            Some(parse_signature_arg(meta.clone())?)
        } else {
            None
        };

        Ok(EventConfig::new(anonymous.is_some(), signature_topic))
    }
}

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
        ));
    }

    let span = s.ast().span();
    let ink_attrs = parse_arg_attrs(&s.ast().attrs)?;
    let config = EventConfig::try_from(ink_attrs.as_slice())?;
    let anonymous = config.anonymous;

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
        return Err(err);
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

    let event_signature_topic = if anonymous {
        None
    } else {
        Some(quote_spanned!(span=>
            .push_topic(Self::SIGNATURE_TOPIC.as_ref())
        ))
    };

    let signature_topic = if !anonymous {
        let event_ident = variant.ast().ident;
        if let Some(sig_arg) = config.signature_topic {
            let bytes = sig_arg.signature_topic();
            quote_spanned!(span=> ::core::option::Option::Some([ #(#bytes),* ]))
        } else {
            let calculated_signature_topic =
                signature_topic(variant.ast().fields, event_ident);
            quote_spanned!(span=> ::core::option::Option::Some(#calculated_signature_topic))
        }
    } else {
        quote_spanned!(span=> ::core::option::Option::None)
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
                format!("Only a single `#[ink({})]` is allowed", path),
            ));
        } else {
            return Err(syn::Error::new(
                a.span(),
                "Unknown ink! attribute at this position".to_string(),
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

/// Parses signature topic from the list of attributes.
///
/// # Errors
/// - Name-value pair is not specified correctly.
/// - Provided value is of wrong format.
/// - Provided hash string is of wrong length.
fn parse_signature_arg(meta: syn::Meta) -> syn::Result<SignatureTopicArg> {
    if let syn::Meta::NameValue(nv) = &meta {
        Ok(SignatureTopicArg::try_from(nv)?)
    } else {
        Err(syn::Error::new(
            meta.span(),
            "Expected to have an argument".to_string(),
        ))
    }
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
