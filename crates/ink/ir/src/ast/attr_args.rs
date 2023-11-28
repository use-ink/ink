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

use super::MetaNameValue;
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    Token,
};

/// The attribute arguments for the configuration of an ink! smart contract.
///
/// For example, the segment `env = ::my::env::Environment`
/// in `#[ink::contract(env = ::my::env::Environment)]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeArgs {
    args: Punctuated<MetaNameValue, Token![,]>,
}

impl quote::ToTokens for AttributeArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.args.to_tokens(tokens)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::MetaValue;
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
                value: MetaValue::Lit(syn::parse_quote! { true }),
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
                value: MetaValue::Lit(syn::parse_quote! { "string literal" }),
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
                value: MetaValue::Path(syn::parse_quote! { MyIdentifier }),
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
                value: MetaValue::Path(syn::parse_quote! { ::this::is::my::Path }),
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
                value: MetaValue::Path(
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
            value: MetaValue::Path(syn::parse_quote! { value }),
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
                    value: MetaValue::Path(syn::parse_quote! { ::root::Path }),
                },
                MetaNameValue {
                    name: syn::parse_quote! { name2 },
                    eq_token: syn::parse_quote! { = },
                    value: MetaValue::Lit(syn::parse_quote! { false }),
                },
                MetaNameValue {
                    name: syn::parse_quote! { name3 },
                    eq_token: syn::parse_quote! { = },
                    value: MetaValue::Lit(syn::parse_quote! { "string literal" }),
                },
                MetaNameValue {
                    name: syn::parse_quote! { name4 },
                    eq_token: syn::parse_quote! { = },
                    value: MetaValue::Lit(syn::parse_quote! { 42 }),
                },
                MetaNameValue {
                    name: syn::parse_quote! { name5 },
                    eq_token: syn::parse_quote! { = },
                    value: MetaValue::Lit(syn::parse_quote! { 7.7 }),
                },
            ])
        )
    }
}
