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

use ink_ir::{
    ast,
    format_err_spanned,
    utils::{
        duplicate_config_err,
        WhitelistedAttributes,
    },
};

/// The End-to-End test configuration.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct E2EConfig {
    /// The set of attributes that can be passed to call builder in the codegen.
    whitelisted_attributes: WhitelistedAttributes,
    /// Additional contracts that have to be built before executing the test.
    additional_contracts: Vec<String>,
}

impl TryFrom<ast::AttributeArgs> for E2EConfig {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut whitelisted_attributes = WhitelistedAttributes::default();
        let mut additional_contracts: Option<(syn::LitStr, ast::MetaNameValue)> = None;

        for arg in args.into_iter() {
            if arg.name.is_ident("keep_attr") {
                whitelisted_attributes.parse_arg_value(&arg)?;
            } else if arg.name.is_ident("additional_contracts") {
                if let Some((_, ast)) = additional_contracts {
                    return Err(duplicate_config_err(
                        ast,
                        arg,
                        "additional_contracts",
                        "e2e test",
                    ))
                }
                if let ast::PathOrLit::Lit(syn::Lit::Str(lit_str)) = &arg.value {
                    additional_contracts = Some((lit_str.clone(), arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a bool literal for `additional_contracts` ink! e2e test configuration argument",
                    ))
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! configuration argument",
                ))
            }
        }
        let additional_contracts = additional_contracts
            .map(|(value, _)| value.value().split(' ').map(String::from).collect())
            .unwrap_or_else(Vec::new);
        Ok(E2EConfig {
            additional_contracts,
            whitelisted_attributes,
        })
    }
}

impl E2EConfig {
    /// Returns a vector of additional contracts that have to be built
    /// and imported before executing the test.
    pub fn additional_contracts(&self) -> Vec<String> {
        self.additional_contracts.clone()
    }
}

/// The environmental types definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    /// The underlying Rust type.
    pub path: syn::Path,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            path: syn::parse_quote! { ::ink_env::DefaultEnvironment },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that the given input configuration attribute argument are converted
    /// into the expected ink! configuration or yields the expected error message.
    fn assert_try_from(
        input: ast::AttributeArgs,
        expected: Result<E2EConfig, &'static str>,
    ) {
        assert_eq!(
            <E2EConfig as TryFrom<ast::AttributeArgs>>::try_from(input)
                .map_err(|err| err.to_string()),
            expected.map_err(ToString::to_string),
        );
    }

    #[test]
    fn empty_config_works() {
        assert_try_from(syn::parse_quote! {}, Ok(E2EConfig::default()))
    }

    #[test]
    fn unknown_arg_fails() {
        assert_try_from(
            syn::parse_quote! { unknown = argument },
            Err("encountered unknown or unsupported ink! configuration argument"),
        );
    }

    #[test]
    fn duplicate_args_fails() {
        assert_try_from(
            syn::parse_quote! {
                additional_contracts = "adder/Cargo.toml",
                additional_contracts = "adder/Cargo.toml",
            },
            Err(
                "encountered duplicate ink! e2e test `additional_contracts` configuration argument",
            ),
        );
    }

    #[test]
    fn keep_attr_works() {
        let mut attrs = WhitelistedAttributes::default();
        attrs.0.insert("foo".to_string(), ());
        attrs.0.insert("bar".to_string(), ());
        assert_try_from(
            syn::parse_quote! {
                keep_attr = "foo, bar"
            },
            Ok(E2EConfig {
                whitelisted_attributes: attrs,
                additional_contracts: Vec::new(),
            }),
        )
    }

    #[test]
    fn keep_attr_invalid_value_fails() {
        assert_try_from(
            syn::parse_quote! { keep_attr = 1u16 },
            Err("expected a string with attributes separated by `,`"),
        );
    }
}
