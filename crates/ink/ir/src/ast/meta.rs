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

use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
};
use quote::ToTokens;
use syn::{
    ext::IdentExt as _,
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    spanned::Spanned,
    LitInt,
    Token,
};

/// Content of a compile-time structured attribute.
///
/// This is a subset of `syn::Meta` that allows the `value` of a name-value pair
/// to be a plain identifier or path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Meta {
    /// A path, like `message`.
    Path(syn::Path),
    /// A name-value pair, like `feature = "nightly"`.
    NameValue(MetaNameValue),
}

impl Parse for Meta {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let path = input.call(parse_meta_path)?;
        if input.peek(Token![=]) {
            MetaNameValue::parse_meta_name_value_after_path(path, input)
                .map(Meta::NameValue)
        } else {
            Ok(Meta::Path(path))
        }
    }
}

impl ToTokens for Meta {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Path(path) => path.to_tokens(tokens),
            Self::NameValue(name_value) => name_value.to_tokens(tokens),
        }
    }
}

/// A name-value pair within an attribute, like `feature = "nightly"`.
///
/// The only difference from `syn::MetaNameValue` is that this additionally
/// allows the `value` to be a plain identifier or path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetaNameValue {
    pub name: syn::Path,
    pub eq_token: syn::token::Eq,
    pub value: PathOrLit,
}

impl Parse for MetaNameValue {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let path = input.call(parse_meta_path)?;
        Self::parse_meta_name_value_after_path(path, input)
    }
}

impl ToTokens for MetaNameValue {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.name.to_tokens(tokens);
        self.eq_token.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

impl MetaNameValue {
    fn parse_meta_name_value_after_path(
        name: syn::Path,
        input: ParseStream,
    ) -> Result<MetaNameValue, syn::Error> {
        let span = name.span();
        Ok(MetaNameValue {
            name,
            eq_token: input.parse().map_err(|_error| {
                format_err!(
                    span,
                    "ink! config options require an argument separated by '='",
                )
            })?,
            value: input.parse()?,
        })
    }
}

/// Either a path or a literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathOrLit {
    Path(syn::Path),
    Lit(syn::Lit),
}

impl Parse for PathOrLit {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        if input.fork().peek(syn::Lit) {
            return input.parse::<syn::Lit>().map(PathOrLit::Lit)
        }
        if input.fork().peek(Ident::peek_any) || input.fork().peek(Token![::]) {
            return input.call(parse_meta_path).map(PathOrLit::Path)
        }
        Err(input.error("cannot parse into either literal or path"))
    }
}

impl ToTokens for PathOrLit {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Lit(lit) => lit.to_tokens(tokens),
            Self::Path(path) => path.to_tokens(tokens),
        }
    }
}

impl PathOrLit {
    /// Returns the value of the literal if it is a boolean literal.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Lit(syn::Lit::Bool(lit_bool)) => Some(lit_bool.value),
            _ => None,
        }
    }

    /// Returns the value of the literal if it is a string literal.
    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::Lit(syn::Lit::Str(lit_str)) => Some(lit_str.value()),
            _ => None,
        }
    }

    /// Returns the the literal if it is an integer literal.
    pub fn as_lit_int(&self) -> Option<&LitInt> {
        match self {
            Self::Lit(syn::Lit::Int(lit_int)) => Some(lit_int),
            _ => None,
        }
    }
}

/// Like [`syn::Path::parse_mod_style`] but accepts keywords in the path.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PathOrLit;
    use quote::quote;

    #[test]
    fn underscore_token_works() {
        assert_eq!(
            syn::parse2::<Meta>(quote! { selector = _ }).unwrap(),
            Meta::NameValue(MetaNameValue {
                name: syn::parse_quote! { selector },
                eq_token: syn::parse_quote! { = },
                value: PathOrLit::Path(syn::Path::from(quote::format_ident!("_"))),
            })
        )
    }
}
