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

use core::convert::TryFrom;

use crate::ast;

/// The ink! configuration.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Config {
    /// If `true` enables the dynamic storage allocator
    /// facilities and code generation of the ink! smart
    /// contract. Does incure some overhead. The default is
    /// `true`.
    storage_alloc: Option<bool>,
    /// If `true` compiles this ink! smart contract always as
    /// if it was a dependency of another smart contract.
    /// This configuration is mainly needed for testing and
    /// the default is `false`.
    as_dependency: Option<bool>,
    /// The environmental types definition.
    ///
    /// This must be a type that implements `ink_core::env::EnvTypes` and can
    /// be used to change the underlying environmental types of an ink! smart
    /// contract.
    env_types: Option<EnvTypes>,
}

impl TryFrom<ast::AttributeArgs> for Config {
    type Error = Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut storage_alloc: Option<(bool, ast::MetaNameValue)> = None;
        let mut as_dependency: Option<(bool, ast::MetaNameValue)> = None;
        let mut env_types: Option<(EnvTypes, ast::MetaNameValue)> = None;
        for arg in args.into_iter() {
            if arg.name.is_ident("storage_alloc") {
                if let Some((_, ast)) = storage_alloc {
                    return Err(Error::duplicate_arg(
                        ast,
                        arg,
                        "found duplicate ink! `storage_allocator` argument",
                    ))
                }
                if let ast::PathOrLit::Lit(syn::Lit::Bool(lit_bool)) = &arg.value {
                    storage_alloc = Some((lit_bool.value, arg))
                } else {
                    return Err(Error::invalid_arg(
                        arg,
                        "expected a bool literal for `storage_allocator` ink! argument",
                    ))
                }
            } else if arg.name.is_ident("compile_as_dependency") {
                if let Some((_, ast)) = as_dependency {
                    return Err(Error::duplicate_arg(
                        ast,
                        arg,
                        "found duplicate ink! `compile_as_dependency` argument",
                    ))
                }
                if let ast::PathOrLit::Lit(syn::Lit::Bool(lit_bool)) = &arg.value {
                    as_dependency = Some((lit_bool.value, arg))
                } else {
                    return Err(Error::invalid_arg(arg, "expected a bool literal for `compile_as_dependency` ink! argument"))
                }
            } else if arg.name.is_ident("env_types") {
                if let Some((_, ast)) = env_types {
                    return Err(Error::duplicate_arg(
                        ast,
                        arg,
                        "found duplicate ink! `env_types` argument",
                    ))
                }
                if let ast::PathOrLit::Path(path) = &arg.value {
                    env_types = Some((
                        EnvTypes {
                            env_types: path.clone(),
                        },
                        arg,
                    ))
                } else {
                    return Err(Error::invalid_arg(
                        arg,
                        "expected a path for `env_types` ink! argument",
                    ))
                }
            } else {
                return Err(Error::invalid_arg(
                    arg,
                    "encountered unknown or unsupported ink! config argument",
                ))
            }
        }
        Ok(Config {
            storage_alloc: storage_alloc.map(|(storage_alloc, _)| storage_alloc),
            as_dependency: as_dependency.map(|(as_dependency, _)| as_dependency),
            env_types: env_types.map(|(env_types, _)| env_types),
        })
    }
}

/// Errors that can be encountered when converting into an ink! configuration.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidArg {
        invalid: ast::MetaNameValue,
        reason: String,
    },
    DuplicateArg {
        fst: ast::MetaNameValue,
        snd: ast::MetaNameValue,
        reason: String,
    },
}

impl Error {
    /// Creates a new error indicating an invalid config argument.
    ///
    /// Use the reason to further specify what happened.
    pub fn invalid_arg<S>(arg: ast::MetaNameValue, reason: S) -> Self
    where
        S: Into<String>,
    {
        Self::InvalidArg {
            invalid: arg,
            reason: reason.into(),
        }
    }

    /// Creates a new error indicating a duplicate config argument.
    ///
    /// Use the reason to further specify what happened.
    pub fn duplicate_arg<S>(
        fst: ast::MetaNameValue,
        snd: ast::MetaNameValue,
        reason: S,
    ) -> Self
    where
        S: Into<String>,
    {
        Self::DuplicateArg {
            fst,
            snd,
            reason: reason.into(),
        }
    }
}

impl Config {
    /// Returns the environmental types definition if specified.
    /// Otherwise returns the default environmental types definition provided
    /// by ink!.
    pub fn env_types(&self) -> EnvTypes {
        self.env_types.as_ref().cloned().unwrap_or_default()
    }

    /// Returns `true` if the dynamic storage allocator facilities are enabled
    /// for the ink! smart contract, `false` otherwise.
    ///
    /// If nothing has been specified returns the default which is `true`.
    pub fn is_storage_allocator_enabled(&self) -> bool {
        self.storage_alloc.unwrap_or(true)
    }

    /// Return `true` if this ink! smart contract shall always be compiled as
    /// if it was a dependency of another smart contract, returns `false`
    /// otherwise.
    ///
    /// If nothing has been specified returns the default which is `false`.
    pub fn is_compile_as_dependency_enabled(&self) -> bool {
        self.as_dependency.unwrap_or(false)
    }
}

/// The environmental types definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvTypes {
    /// The underlying Rust type.
    env_types: syn::Path,
}

impl Default for EnvTypes {
    fn default() -> Self {
        Self {
            env_types: syn::parse_quote! { ::ink_core::env::DefaultEnvTypes },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_works() {
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(syn::parse_quote! {}),
            Ok(Config::default()),
        )
    }

    #[test]
    fn storage_alloc_works() {
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(syn::parse_quote! {
                storage_alloc = true
            }),
            Ok(Config {
                storage_alloc: Some(true),
                as_dependency: None,
                env_types: None,
            }),
        )
    }

    #[test]
    fn storage_alloc_invalid_value_fails() {
        let invalid = syn::parse_quote! { storage_alloc = "invalid" };
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(syn::parse_quote! {
                #invalid
            }),
            Err(Error::invalid_arg(
                invalid,
                "expected a bool literal for `storage_allocator` ink! argument"
            ))
        )
    }

    #[test]
    fn as_dependency_works() {
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(syn::parse_quote! {
                compile_as_dependency = false
            }),
            Ok(Config {
                storage_alloc: None,
                as_dependency: Some(false),
                env_types: None,
            }),
        )
    }

    #[test]
    fn as_dependency_invalid_value_fails() {
        let invalid = syn::parse_quote! { compile_as_dependency = "invalid" };
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(syn::parse_quote! {
                #invalid
            }),
            Err(Error::invalid_arg(
                invalid,
                "expected a bool literal for `compile_as_dependency` ink! argument"
            ))
        )
    }

    #[test]
    fn env_types_works() {
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(syn::parse_quote! {
                env_types = ::my::env::Types
            }),
            Ok(Config {
                storage_alloc: None,
                as_dependency: None,
                env_types: Some(EnvTypes {
                    env_types: syn::parse_quote! { ::my::env::Types }
                }),
            }),
        )
    }

    #[test]
    fn env_types_invalid_value_fails() {
        let invalid = syn::parse_quote! { env_types = "invalid" };
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(syn::parse_quote! {
                #invalid
            }),
            Err(Error::invalid_arg(
                invalid,
                "expected a path for `env_types` ink! argument"
            ))
        )
    }

    #[test]
    fn unknown_arg_fails() {
        let unknown = syn::parse_quote! { unknown = argument };
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(syn::parse_quote! {
                #unknown
            }),
            Err(Error::invalid_arg(
                unknown,
                "encountered unknown or unsupported ink! config argument"
            ))
        )
    }

    #[test]
    fn duplicate_args_fails() {
        let fst = syn::parse_quote! { env_types = ::my::env::Types };
        let snd = syn::parse_quote! { env_types = ::my::other::env::Types };
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(
                syn::parse_quote! { #fst, #snd }
            ),
            Err(Error::duplicate_arg(
                fst,
                snd,
                "found duplicate ink! `env_types` argument"
            ))
        )
    }
}
