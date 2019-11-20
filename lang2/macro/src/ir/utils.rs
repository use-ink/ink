// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

//! Contains general utilities for the ink! IR module.

use proc_macro2::Span;
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    spanned::Spanned,
    Result,
};

use crate::ir;

/// An unsuffixed integer literal: `0` or `42` or `1337`
#[derive(Debug, Clone)]
pub struct UnsuffixedLitInt {
    pub(crate) lit_int: syn::LitInt,
}

impl Spanned for UnsuffixedLitInt {
    fn span(&self) -> Span {
        self.lit_int.span()
    }
}

impl Parse for UnsuffixedLitInt {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit_int: syn::LitInt = input.parse()?;
        if lit_int.suffix() != "" {
            bail!(lit_int, "integer suffixes are not allowed here",)
        }
        Ok(Self { lit_int })
    }
}

/// Returns `true` if the given attribute is any of `#[ink(..)]`.
pub fn is_ink_attribute(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("ink")
}

/// Yields back all non-`#[ink(..)]` attributes if any.
pub fn filter_non_ink_attributes<'a, I>(
    attrs: I,
) -> impl Iterator<Item = &'a syn::Attribute> + 'a
where
    I: IntoIterator<Item = &'a syn::Attribute> + 'a,
{
    attrs.into_iter().filter(|attr| !is_ink_attribute(attr))
}

/// Yields back the filtered `#[ink(..)]` markers if any.
pub fn filter_ink_attributes<'a, I>(
    attrs: I,
) -> impl Iterator<Item = &'a syn::Attribute> + 'a
where
    I: IntoIterator<Item = &'a syn::Attribute> + 'a,
{
    attrs.into_iter().filter(|attr| is_ink_attribute(attr))
}

/// Yields back the filtered `#[ink(..)]` markers converted into their ink! form if any.
pub fn filter_map_ink_attributes<'a, I>(attrs: I) -> impl Iterator<Item = ir::Marker>
where
    I: IntoIterator<Item = &'a syn::Attribute> + 'a,
{
    use core::convert::TryFrom as _;
    attrs
        .into_iter()
        .cloned()
        .filter_map(|attr| ir::Marker::try_from(attr).ok())
}

/// Returns `true` if the attributes contain any `#[ink(..)]` markers.
pub fn has_ink_attributes<'a, I>(attrs: I) -> bool
where
    I: IntoIterator<Item = &'a syn::Attribute> + 'a,
{
    filter_ink_attributes(attrs).count() > 0
}

/// Filters the given attributes for `#[doc(..)]` attributes
/// and trims them to human-readable documentation strings.
///
/// # Note
///
/// This is mainly used in the ABI generation routines.
pub fn filter_map_trimmed_doc_strings<'a, I>(
    attrs: I,
) -> impl Iterator<Item = String> + 'a
where
    I: IntoIterator<Item = &'a syn::Attribute>,
    <I as IntoIterator>::IntoIter: 'a,
{
    attrs
        .into_iter()
        .filter(move |attr| {
            attr.style == syn::AttrStyle::Outer && attr.path.is_ident("doc")
        })
        .map(to_trimmed_doc_string)
}

/// Trims a doc string obtained from an attribute token stream into the actual doc string.
///
/// Practically speaking this method removes the trailing start `" = \""` and end `\"`
/// of documentation strings coming from Syn attribute token streams.
pub fn to_trimmed_doc_string(attr: &syn::Attribute) -> String {
    attr.tokens
        .to_string()
        .trim_start_matches('=')
        .trim_start()
        .trim_start_matches('r')
        .trim_start_matches('"')
        .trim_end_matches('"')
        .trim()
        .into()
}
