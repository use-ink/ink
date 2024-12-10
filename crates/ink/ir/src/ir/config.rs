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

use crate::{
    ast,
    utils::{
        duplicate_config_err,
        WhitelistedAttributes,
    },
};

/// The ink! configuration.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Config {
    /// The environmental types definition.
    ///
    /// This must be a type that implements `ink_env::Environment` and can
    /// be used to change the underlying environmental types of an ink! smart
    /// contract.
    env: Option<Environment>,
    /// todo: docs
    abi_encoding: AbiEncoding,
    /// The set of attributes that can be passed to call builder in the codegen.
    whitelisted_attributes: WhitelistedAttributes,
}

impl TryFrom<ast::AttributeArgs> for Config {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut env: Option<(Environment, ast::MetaNameValue)> = None;
        let mut whitelisted_attributes = WhitelistedAttributes::default();
        let mut abi_encoding: Option<(AbiEncoding, ast::MetaNameValue)> = None;

        for arg in args.into_iter() {
            if arg.name().is_ident("env") {
                if let Some((_, ast)) = env {
                    return Err(duplicate_config_err(ast, arg, "env", "contract"));
                }
                let env_info = arg
                    .name_value()
                    .zip(arg.value().and_then(ast::MetaValue::as_path));
                if let Some((name_value, path)) = env_info {
                    env = Some((Environment { path: path.clone() }, name_value.clone()))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a path value for `env` ink! configuration argument",
                    ));
                }
            } else if arg.name().is_ident("abi_encoding") {
                if let Some((_, ast)) = abi_encoding {
                    return Err(duplicate_config_err(
                        ast,
                        arg,
                        "abi_encoding",
                        "contract",
                    ));
                }
                let encoding = arg
                    .name_value()
                    .zip(arg.value().and_then(ast::MetaValue::as_string));
                if let Some((name_value, path)) = encoding {
                    let encoding = match path.as_str() {
                        "scale" => AbiEncoding::Scale,
                        "rlp" => AbiEncoding::Rlp,
                        "all" => AbiEncoding::All,
                        _ => {
                            return Err(format_err_spanned!(
                                arg,
                                "expected one of `scale`, `rlp` or `all` for `abi_encoding` ink! configuration argument",
                            ));
                        }
                    };
                    abi_encoding = Some((encoding, name_value.clone()))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string value for `abi_encoding` ink! configuration argument",
                    ));
                }
            } else if arg.name().is_ident("keep_attr") {
                if let Some(name_value) = arg.name_value() {
                    whitelisted_attributes.parse_arg_value(name_value)?;
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string literal value for `keep_attr` ink! configuration argument",
                    ));
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! configuration argument",
                ));
            }
        }
        Ok(Config {
            env: env.map(|(value, _)| value),
            abi_encoding: abi_encoding
                .map_or(AbiEncoding::default(), |(encoding, _)| encoding),
            whitelisted_attributes,
        })
    }
}

impl Config {
    /// Returns the environmental types definition if specified.
    /// Otherwise returns the default environmental types definition provided
    /// by ink!.
    pub fn env(&self) -> syn::Path {
        self.env
            .as_ref()
            .map(|env| &env.path)
            .cloned()
            .unwrap_or(Environment::default().path)
    }

    pub fn abi_encoding(&self) -> &AbiEncoding {
        &self.abi_encoding
    }

    /// Return set of attributes that can be passed to call builder in the codegen.
    pub fn whitelisted_attributes(&self) -> &WhitelistedAttributes {
        &self.whitelisted_attributes
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
            path: syn::parse_quote! { ::ink::env::DefaultEnvironment },
        }
    }
}

/// Which format is used for ABI encoding.
#[derive(Debug, Clone, PartialEq, Eq, Default)]

pub enum AbiEncoding {
    /// Scale codec, the default.
    #[default]
    Scale,
    /// RLP codec, useful for compatibility with Solidity contracts.
    Rlp,
    /// Support both Scale and RLP encoding for each contract entry point.
    All,
}

impl AbiEncoding {
    pub fn is_rlp(&self) -> bool {
        matches!(self, Self::Rlp | Self::All)
    }

    pub fn is_scale(&self) -> bool {
        matches!(self, Self::Scale | Self::All)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that the given input configuration attribute argument are converted
    /// into the expected ink! configuration or yields the expected error message.
    fn assert_try_from(
        input: ast::AttributeArgs,
        expected: Result<Config, &'static str>,
    ) {
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(input)
                .map_err(|err| err.to_string()),
            expected.map_err(ToString::to_string),
        );
    }

    #[test]
    fn empty_config_works() {
        assert_try_from(syn::parse_quote! {}, Ok(Config::default()))
    }

    #[test]
    fn env_works() {
        assert_try_from(
            syn::parse_quote! {
                env = ::my::env::Types
            },
            Ok(Config {
                env: Some(Environment {
                    path: syn::parse_quote! { ::my::env::Types },
                }),
                whitelisted_attributes: Default::default(),
                abi_encoding: Default::default(),
            }),
        )
    }

    #[test]
    fn env_invalid_value_fails() {
        assert_try_from(
            syn::parse_quote! { env = "invalid" },
            Err("expected a path value for `env` ink! configuration argument"),
        );
    }

    #[test]
    fn env_missing_value_fails() {
        assert_try_from(
            syn::parse_quote! { env },
            Err("expected a path value for `env` ink! configuration argument"),
        );
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
                env = ::my::env::Types,
                env = ::my::other::env::Types,
            },
            Err("encountered duplicate ink! contract `env` configuration argument"),
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
            Ok(Config {
                env: None,
                abi_encoding: Default::default(),
                whitelisted_attributes: attrs,
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

    #[test]
    fn keep_attr_missing_value_fails() {
        assert_try_from(
            syn::parse_quote! { keep_attr },
            Err("expected a string literal value for `keep_attr` ink! configuration argument"),
        );
    }
}
