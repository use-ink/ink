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

use super::Selector;
use crate::{
    ast::{
        self,
        MetaNameValue,
        MetaValue,
    },
    error::ExtError as _,
    format_err,
};
use proc_macro2::Span;
use std::collections::HashMap;
use syn::spanned::Spanned;

/// Ensures that the given visibility is `pub` and otherwise returns an appropriate error.
///
/// # Note
///
/// The `name` parameter is given to improve the resulting error message. It denotes the
/// entity which cannot have non-public visibility.
pub fn ensure_pub_visibility(
    name: &str,
    parent_span: Span,
    vis: &syn::Visibility,
) -> Result<(), syn::Error> {
    let bad_visibility = match vis {
        syn::Visibility::Inherited => Some(parent_span),
        syn::Visibility::Restricted(vis_restricted) => Some(vis_restricted.span()),
        syn::Visibility::Public(_) => None,
    };
    if let Some(bad_visibility) = bad_visibility {
        return Err(format_err!(
            bad_visibility,
            "non `pub` ink! {} are not supported",
            name
        ))
    }
    Ok(())
}

/// Returns a local ID unique to the ink! trait definition for the identifier.
///
/// # Note
///
/// - The returned value is equal to the selector of the message identifier.
/// - Used from within ink! trait definitions as well as ink! trait implementation blocks.
pub fn local_message_id(ident: &syn::Ident) -> u32 {
    let input = ident.to_string().into_bytes();
    let selector = Selector::compute(&input);
    selector.into_be_u32()
}

/// The set of attributes that can be passed to call builder or call forwarder in the
/// codegen.
#[derive(Debug, PartialEq, Eq)]
pub struct WhitelistedAttributes(pub HashMap<String, ()>);

impl Default for WhitelistedAttributes {
    fn default() -> Self {
        Self(HashMap::from([
            // Conditional compilation
            ("cfg".to_string(), ()),
            ("cfg_attr".to_string(), ()),
            // Diagnostics
            ("allow".to_string(), ()),
            ("warn".to_string(), ()),
            ("deny".to_string(), ()),
            ("forbid".to_string(), ()),
            ("deprecated".to_string(), ()),
            ("must_use".to_string(), ()),
            // Documentation
            ("doc".to_string(), ()),
            // Formatting
            ("rustfmt".to_string(), ()),
        ]))
    }
}

impl WhitelistedAttributes {
    /// Parses the `MetaNameValue` argument of `keep_attr` attribute. If the argument has
    /// a correct format `"foo, bar"` then `foo`, `bar` will be included in
    /// the whitelist of attributes. Else error about parsing will be returned.
    pub fn parse_arg_value(&mut self, arg: &MetaNameValue) -> Result<(), syn::Error> {
        if let ast::MetaValue::Lit(syn::Lit::Str(attributes)) = &arg.value {
            attributes.value().split(',').for_each(|attribute| {
                self.0.insert(attribute.trim().to_string(), ());
            });
            Ok(())
        } else {
            Err(format_err_spanned!(
                arg,
                "expected a string with attributes separated by `,`",
            ))
        }
    }

    /// Returns the filtered input vector of whitelisted attributes.
    /// All not whitelisted attributes are removed.
    pub fn filter_attr(&self, attrs: Vec<syn::Attribute>) -> Vec<syn::Attribute> {
        attrs
            .into_iter()
            .filter(|attr| {
                if let Some(ident) = attr.path().get_ident() {
                    self.0.contains_key(&ident.to_string())
                } else {
                    false
                }
            })
            .collect()
    }
}

/// Return an error to notify about duplicate ink! configuration arguments.
pub fn duplicate_config_err<F, S>(
    first: F,
    second: S,
    name: &str,
    ink_attr: &str,
) -> syn::Error
where
    F: Spanned,
    S: Spanned,
{
    format_err!(
        second.span(),
        "encountered duplicate ink! {} `{}` configuration argument",
        ink_attr,
        name,
    )
    .into_combine(format_err!(
        first.span(),
        "first `{}` configuration argument here",
        name
    ))
}

/// Finds the salt of a struct, enum or union.
/// The salt is any generic that has bound `StorageKey`.
/// In most cases it is the parent storage key or the auto-generated storage key.
pub fn find_storage_key_salt(input: &syn::DeriveInput) -> Option<syn::TypeParam> {
    input.generics.params.iter().find_map(|param| {
        if let syn::GenericParam::Type(type_param) = param
            && let Some(syn::TypeParamBound::Trait(trait_bound)) =
                type_param.bounds.first()
        {
            let segments = &trait_bound.path.segments;
            if let Some(last) = segments.last()
                && last.ident == "StorageKey"
            {
                return Some(type_param.clone())
            }
        }
        None
    })
}

/// Extracts `cfg` attributes from the given set of attributes
pub fn extract_cfg_attributes(
    attrs: &[syn::Attribute],
    span: Span,
) -> Vec<proc_macro2::TokenStream> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident(super::CFG_IDENT))
        .map(|a| quote::quote_spanned!(span=> #a ))
        .collect()
}

/// Extracts `cfg` attributes from the given set of attributes
pub fn extract_cfg_syn_attributes(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident(super::CFG_IDENT))
        .cloned()
        .collect()
}

/// Returns `syn::LitStr` value if it's an "identifier-like" string.
///
/// # Note
///
/// The string is considered to be "identifier-like" if:
/// - It begins with an alphabetic character, underscore or dollar sign
/// - It only contains alphanumeric characters, underscores and dollar signs
pub fn extract_name_override(value: &MetaValue, span: Span) -> syn::Result<syn::LitStr> {
    if let Some(lit_str) = value.as_lit_string() {
        let name = lit_str.value();
        if !name
            .chars()
            .next()
            .map(|c| c.is_alphabetic() || c == '$' || c == '_')
            .unwrap_or(false)
        {
            return Err(format_err_spanned!(
                lit_str,
                "`name` attribute argument value must begin with an \
                alphabetic character, underscore or dollar sign",
            ));
        }

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '$' || c == '_')
        {
            return Err(format_err_spanned!(
                lit_str,
                "`name` attribute argument value can only contain \
                alphanumeric characters, underscores and dollar signs",
            ));
        }

        Ok(lit_str.clone())
    } else {
        Err(syn::Error::new(
            span,
            "expected a string literal value for `name` \
            attribute argument",
        ))
    }
}
