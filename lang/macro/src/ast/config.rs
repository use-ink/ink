// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use proc_macro2::Ident;
use syn::{
    ext::IdentExt as _,
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
};

/// The attribute arguments for the configuration of an ink! smart contract.
///
/// These are the segments `version = "0.1.0"` and `env_types = DefaultEnvTypes`
/// in `#[ink::contract(version = "0.1.0", env_types = DefaultEnvTypes)]`.
pub type AttributeArgs = Vec<MetaNameValue>;

/// A name-value pair within an attribute, like feature = "nightly".
///
/// The only difference from `syn::MetaNameValue` is that this additionally
/// allows the `value` to be a plain identifier or path.
pub struct MetaNameValue {
    pub name: syn::Path,
    pub eq_token: syn::token::Eq,
    pub value: PathOrLit,
}

/// Either a path or a literal.
pub enum PathOrLit {
    Path(syn::Path),
    Lit(syn::Lit),
}

impl Parse for MetaNameValue {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let path = input.call(Self::parse_meta_path)?;
        Self::parse_meta_name_value_after_path(path, input)
    }
}

impl MetaNameValue {
    /// Like Path::parse_mod_style but accepts keywords in the path.
    ///
    /// # Note
    ///
    /// This code was taken from the `syn` implementation for a very similar
    /// syntactical pattern.
    fn parse_meta_path(input: ParseStream) -> Result<syn::Path, syn::Error> {
        Ok(syn::Path {
            leading_colon: input.parse()?,
            segments: {
                let mut segments = Punctuated::new();
                while input.peek(Ident::peek_any) {
                    let ident = Ident::parse_any(input)?;
                    segments.push_value(syn::PathSegment::from(ident));
                    if !input.peek(syn::Token![::]) {
                        break
                    }
                    let punct = input.parse()?;
                    segments.push_punct(punct);
                }
                if segments.is_empty() {
                    return Err(input.error("expected path"))
                } else if segments.trailing_punct() {
                    return Err(input.error("expected path segment"))
                }
                segments
            },
        })
    }

    fn parse_meta_name_value_after_path(
        name: syn::Path,
        input: ParseStream,
    ) -> Result<MetaNameValue, syn::Error> {
        Ok(MetaNameValue {
            name,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for PathOrLit {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        if let Ok(lit) = input.fork().parse::<syn::Lit>() {
            return Ok(PathOrLit::Lit(lit))
        }
        if let Ok(path) = input.parse::<syn::Path>() {
            return Ok(PathOrLit::Path(path))
        }
        Err(input.error("cannot parse into either literal or path"))
    }
}

impl PathOrLit {
    /// Determines whether this is a path of length 1 equal to the given ident.
    /// For them to compare equal, it must be the case that:
    ///
    /// - the path has no leading colon,
    /// - the number of path segments is 1,
    /// - the first path segment has no angle bracketed or parenthesized path arguments, and
    /// - the ident of the first path segment is equal to the given one.
    pub fn is_ident<I: ?Sized>(&self, ident: &I) -> bool
    where
        Ident: PartialEq<I>,
    {
        if let Self::Path(path) = self {
            return path.is_ident(ident)
        }
        false
    }
}
