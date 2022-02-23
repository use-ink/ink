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

use crate::{
    ast,
    ast::MetaNameValue,
    error::ExtError as _,
};
use core::convert::TryFrom;
use std::collections::HashMap;
use syn::spanned::Spanned;

/// The ink! configuration.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Config {
    /// If `true` enables the dynamic storage allocator
    /// facilities and code generation of the ink! smart
    /// contract. Does incur some overhead. The default is
    /// `true`.
    dynamic_storage_allocator: Option<bool>,
    /// If `true` compiles this ink! smart contract always as
    /// if it was a dependency of another smart contract.
    /// This configuration is mainly needed for testing and
    /// the default is `false`.
    as_dependency: Option<bool>,
    /// The environmental types definition.
    ///
    /// This must be a type that implements `ink_env::Environment` and can
    /// be used to change the underlying environmental types of an ink! smart
    /// contract.
    env: Option<Environment>,
    /// The set of attributes that can be passed to call builder in the codegen.
    whitelisted_attributes: WhitelistedAttributes,
}

/// The set of attributes that can be passed to call builder or call forwarder in the codegen.
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
        ]))
    }
}

impl WhitelistedAttributes {
    pub fn parse_arg_value(&mut self, arg: &MetaNameValue) -> Result<(), syn::Error> {
        return if let ast::PathOrLit::Lit(syn::Lit::Str(attributes)) = &arg.value {
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

    pub fn filter_attr(&self, attrs: Vec<syn::Attribute>) -> Vec<syn::Attribute> {
        attrs
            .into_iter()
            .filter(|attr| {
                if let Some(ident) = attr.path.get_ident() {
                    self.0.contains_key(&ident.to_string())
                } else {
                    false
                }
            })
            .collect()
    }
}

/// Return an error to notify about duplicate ink! configuration arguments.
fn duplicate_config_err<F, S>(fst: F, snd: S, name: &str) -> syn::Error
where
    F: Spanned,
    S: Spanned,
{
    format_err!(
        snd.span(),
        "encountered duplicate ink! `{}` configuration argument",
        name,
    )
    .into_combine(format_err!(
        fst.span(),
        "first `{}` configuration argument here",
        name
    ))
}

impl TryFrom<ast::AttributeArgs> for Config {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut dynamic_storage_allocator: Option<(bool, ast::MetaNameValue)> = None;
        let mut as_dependency: Option<(bool, ast::MetaNameValue)> = None;
        let mut env: Option<(Environment, ast::MetaNameValue)> = None;
        let mut whitelisted_attributes = WhitelistedAttributes::default();

        for arg in args.into_iter() {
            if arg.name.is_ident("dynamic_storage_allocator") {
                if let Some((_, ast)) = dynamic_storage_allocator {
                    return Err(duplicate_config_err(
                        ast,
                        arg,
                        "dynamic_storage_allocator",
                    ))
                }
                if let ast::PathOrLit::Lit(syn::Lit::Bool(lit_bool)) = &arg.value {
                    dynamic_storage_allocator = Some((lit_bool.value, arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a bool literal for `dynamic_storage_allocator` ink! configuration argument",
                    ))
                }
            } else if arg.name.is_ident("compile_as_dependency") {
                if let Some((_, ast)) = as_dependency {
                    return Err(duplicate_config_err(ast, arg, "compile_as_dependency"))
                }
                if let ast::PathOrLit::Lit(syn::Lit::Bool(lit_bool)) = &arg.value {
                    as_dependency = Some((lit_bool.value, arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a bool literal for `compile_as_dependency` ink! configuration argument",
                    ))
                }
            } else if arg.name.is_ident("env") {
                if let Some((_, ast)) = env {
                    return Err(duplicate_config_err(ast, arg, "env"))
                }
                if let ast::PathOrLit::Path(path) = &arg.value {
                    env = Some((Environment { path: path.clone() }, arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a path for `env` ink! configuration argument",
                    ))
                }
            } else if arg.name.is_ident("keep_attr") {
                if let Err(err) = whitelisted_attributes.parse_arg_value(&arg) {
                    return Err(err)
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! configuration argument",
                ))
            }
        }
        Ok(Config {
            dynamic_storage_allocator: dynamic_storage_allocator.map(|(value, _)| value),
            as_dependency: as_dependency.map(|(value, _)| value),
            env: env.map(|(value, _)| value),
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

    /// Returns `true` if the dynamic storage allocator facilities are enabled
    /// for the ink! smart contract, `false` otherwise.
    ///
    /// If nothing has been specified returns the default which is `false`.
    pub fn is_dynamic_storage_allocator_enabled(&self) -> bool {
        self.dynamic_storage_allocator.unwrap_or(false)
    }

    /// Return `true` if this ink! smart contract shall always be compiled as
    /// if it was a dependency of another smart contract, returns `false`
    /// otherwise.
    ///
    /// If nothing has been specified returns the default which is `false`.
    pub fn is_compile_as_dependency_enabled(&self) -> bool {
        self.as_dependency.unwrap_or(false)
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
    fn storage_alloc_works() {
        assert_try_from(
            syn::parse_quote! {
                dynamic_storage_allocator = true
            },
            Ok(Config {
                dynamic_storage_allocator: Some(true),
                as_dependency: None,
                env: None,
                whitelisted_attributes: Default::default(),
            }),
        )
    }

    #[test]
    fn storage_alloc_invalid_value_fails() {
        assert_try_from(
            syn::parse_quote! { dynamic_storage_allocator = "invalid" },
            Err("expected a bool literal for `dynamic_storage_allocator` ink! configuration argument"),
        )
    }

    #[test]
    fn as_dependency_works() {
        assert_try_from(
            syn::parse_quote! {
                compile_as_dependency = false
            },
            Ok(Config {
                dynamic_storage_allocator: None,
                as_dependency: Some(false),
                env: None,
                whitelisted_attributes: Default::default(),
            }),
        )
    }

    #[test]
    fn as_dependency_invalid_value_fails() {
        assert_try_from(
            syn::parse_quote! { compile_as_dependency = "invalid" },
            Err(
                "expected a bool literal for `compile_as_dependency` ink! configuration argument"
            )
        )
    }

    #[test]
    fn env_works() {
        assert_try_from(
            syn::parse_quote! {
                env = ::my::env::Types
            },
            Ok(Config {
                dynamic_storage_allocator: None,
                as_dependency: None,
                env: Some(Environment {
                    path: syn::parse_quote! { ::my::env::Types },
                }),
                whitelisted_attributes: Default::default(),
            }),
        )
    }

    #[test]
    fn env_invalid_value_fails() {
        assert_try_from(
            syn::parse_quote! { env = "invalid" },
            Err("expected a path for `env` ink! configuration argument"),
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
            Err("encountered duplicate ink! `env` configuration argument"),
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
                dynamic_storage_allocator: None,
                as_dependency: None,
                env: None,
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
}
