// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
    Token,
};

/// The attribute arguments for the configuration of an ink! smart contract.
///
/// These are the segments `env = ::my::env::Environment` and `compile_as_dependency = true`
/// in `#[ink::contract(env = ::my::env::Environment, compile_as_dependency = true`.
#[derive(Debug, PartialEq, Eq)]
pub struct AttributeArgs {
    args: Punctuated<MetaNameValue, Token![,]>,
}

/// A name-value pair within an attribute, like `feature = "nightly"`.
///
/// The only difference from `syn::MetaNameValue` is that this additionally
/// allows the `value` to be a plain identifier or path.
#[derive(Debug, PartialEq, Eq)]
pub struct MetaNameValue {
    pub name: syn::Path,
    pub eq_token: syn::token::Eq,
    pub value: PathOrLit,
}

/// Either a path or a literal.
#[derive(Debug, PartialEq, Eq)]
pub enum PathOrLit {
    Path(syn::Path),
    Lit(syn::Lit),
}

impl IntoIterator for AttributeArgs {
    type Item = MetaNameValue;
    type IntoIter = syn::punctuated::IntoIter<MetaNameValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}

impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        Ok(Self {
            args: Punctuated::parse_terminated(input)?,
        })
    }
}

impl Parse for MetaNameValue {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let path = input.call(Self::parse_meta_path)?;
        Self::parse_meta_name_value_after_path(path, input)
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

impl ToTokens for MetaNameValue {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.name.to_tokens(tokens);
        self.eq_token.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

impl MetaNameValue {
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
        if input.fork().peek(syn::Lit) {
            return input.parse::<syn::Lit>().map(PathOrLit::Lit)
        }
        if input.fork().peek(Ident::peek_any) || input.fork().peek(Token![::]) {
            return input.parse::<syn::Path>().map(PathOrLit::Path)
        }
        Err(input.error("cannot parse into either literal or path"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    impl AttributeArgs {
        /// Creates a new attribute argument list from the given arguments.
        pub fn new<I>(args: I) -> Self
        where
            I: IntoIterator<Item = MetaNameValue>,
        {
            Self {
                args: args.into_iter().collect(),
            }
        }
    }

    #[test]
    fn empty_works() {
        assert_eq!(
            syn::parse2::<AttributeArgs>(quote! {}).unwrap(),
            AttributeArgs::new(vec![])
        )
    }

    #[test]
    fn literal_bool_value_works() {
        assert_eq!(
            syn::parse2::<AttributeArgs>(quote! { name = true }).unwrap(),
            AttributeArgs::new(vec![MetaNameValue {
                name: syn::parse_quote! { name },
                eq_token: syn::parse_quote! { = },
                value: PathOrLit::Lit(syn::parse_quote! { true }),
            }])
        )
    }

    #[test]
    fn literal_str_value_works() {
        assert_eq!(
            syn::parse2::<AttributeArgs>(quote! { name = "string literal" }).unwrap(),
            AttributeArgs::new(vec![MetaNameValue {
                name: syn::parse_quote! { name },
                eq_token: syn::parse_quote! { = },
                value: PathOrLit::Lit(syn::parse_quote! { "string literal" }),
            }])
        )
    }

    #[test]
    fn ident_value_works() {
        assert_eq!(
            syn::parse2::<AttributeArgs>(quote! { name = MyIdentifier }).unwrap(),
            AttributeArgs::new(vec![MetaNameValue {
                name: syn::parse_quote! { name },
                eq_token: syn::parse_quote! { = },
                value: PathOrLit::Path(syn::parse_quote! { MyIdentifier }),
            }])
        )
    }

    #[test]
    fn root_path_value_works() {
        assert_eq!(
            syn::parse2::<AttributeArgs>(quote! { name = ::this::is::my::Path }).unwrap(),
            AttributeArgs::new(vec![MetaNameValue {
                name: syn::parse_quote! { name },
                eq_token: syn::parse_quote! { = },
                value: PathOrLit::Path(syn::parse_quote! { ::this::is::my::Path }),
            }])
        )
    }

    #[test]
    fn relative_path_value_works() {
        assert_eq!(
            syn::parse2::<AttributeArgs>(quote! { name = this::is::my::relative::Path })
                .unwrap(),
            AttributeArgs::new(vec![MetaNameValue {
                name: syn::parse_quote! { name },
                eq_token: syn::parse_quote! { = },
                value: PathOrLit::Path(
                    syn::parse_quote! { this::is::my::relative::Path }
                ),
            }])
        )
    }

    #[test]
    fn trailing_comma_works() {
        let mut expected_args = Punctuated::new();
        expected_args.push_value(MetaNameValue {
            name: syn::parse_quote! { name },
            eq_token: syn::parse_quote! { = },
            value: PathOrLit::Path(syn::parse_quote! { value }),
        });
        expected_args.push_punct(<Token![,]>::default());
        assert_eq!(
            syn::parse2::<AttributeArgs>(quote! { name = value, }).unwrap(),
            AttributeArgs {
                args: expected_args,
            }
        )
    }

    #[test]
    fn many_mixed_works() {
        assert_eq!(
            syn::parse2::<AttributeArgs>(quote! {
                name1 = ::root::Path,
                name2 = false,
                name3 = "string literal",
                name4 = 42,
                name5 = 7.7
            })
            .unwrap(),
            AttributeArgs::new(vec![
                MetaNameValue {
                    name: syn::parse_quote! { name1 },
                    eq_token: syn::parse_quote! { = },
                    value: PathOrLit::Path(syn::parse_quote! { ::root::Path }),
                },
                MetaNameValue {
                    name: syn::parse_quote! { name2 },
                    eq_token: syn::parse_quote! { = },
                    value: PathOrLit::Lit(syn::parse_quote! { false }),
                },
                MetaNameValue {
                    name: syn::parse_quote! { name3 },
                    eq_token: syn::parse_quote! { = },
                    value: PathOrLit::Lit(syn::parse_quote! { "string literal" }),
                },
                MetaNameValue {
                    name: syn::parse_quote! { name4 },
                    eq_token: syn::parse_quote! { = },
                    value: PathOrLit::Lit(syn::parse_quote! { 42 }),
                },
                MetaNameValue {
                    name: syn::parse_quote! { name5 },
                    eq_token: syn::parse_quote! { = },
                    value: PathOrLit::Lit(syn::parse_quote! { 7.7 }),
                },
            ])
        )
    }
}
