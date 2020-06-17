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
use regex::Regex;

/// The ink! configuration.
pub struct Config {
    /// The version of the ink! smart contract.
    version: InkVersion,
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

impl TryFrom<syn::AttributeArgs> for Config {
    type Error = Error;

    fn try_from(args: syn::AttributeArgs) -> Result<Self, Self::Error> {
        let mut version: Option<(InkVersion, syn::NestedMeta)> = None;
        let mut storage_alloc: Option<(bool, syn::NestedMeta)> = None;
        let mut as_dependency: Option<(bool, syn::NestedMeta)> = None;
        let mut env_types: Option<(EnvTypes, syn::NestedMeta)> = None;
        for arg in args.into_iter() {
            match arg {
                syn::NestedMeta::Lit(_) => {
                    return Err(Error::invalid_arg(
                        arg,
                        "encountered invalid or unknown literal ink! argument",
                    ))
                }
                syn::NestedMeta::Meta(ref meta) => {
                    match meta {
                        syn::Meta::NameValue(name_value) => {
                            if name_value.path.is_ident("version") {
                                if let Some((_, ast)) = version {
                                    return Err(Error::duplicate_arg(
                                        ast,
                                        arg,
                                        "found duplicate ink! version argument",
                                    ))
                                }
                                version = Some((
                                    <InkVersion as TryFrom<_>>::try_from(name_value)?,
                                    arg,
                                ));
                            } else if name_value.path.is_ident("storage_alloc") {
                                if let Some((_, ast)) = storage_alloc {
                                    return Err(Error::duplicate_arg(
                                        ast,
                                        arg,
                                        "found duplicate ink! `storage_allocator` argument",
                                    ))
                                }
                                match &name_value.lit {
                                    syn::Lit::Bool(lit_bool) => {
                                        storage_alloc = Some((lit_bool.value, arg))
                                    }
                                    _ => {
                                        return Err(Error::invalid_arg(arg, "expected a bool literal for `storage_allocator` ink! argument"))
                                    }
                                }
                            } else if name_value.path.is_ident("compile_as_dependency") {
                                if let Some((_, ast)) = as_dependency {
                                    return Err(Error::duplicate_arg(
                                        ast,
                                        arg,
                                        "found duplicate ink! `compile_as_dependency` argument",
                                    ))
                                }
                                match &name_value.lit {
                                    syn::Lit::Bool(lit_bool) => {
                                        as_dependency = Some((lit_bool.value, arg))
                                    }
                                    _ => {
                                        return Err(Error::invalid_arg(arg, "expected a bool literal for `compile_as_dependency` ink! argument"))
                                    }
                                }
                            } else if name_value.path.is_ident("env_types") {
                                if let Some((_, ast)) = env_types {
                                    return Err(Error::duplicate_arg(
                                        ast,
                                        arg,
                                        "found duplicate ink! `env_types` argument",
                                    ))
                                }
                                env_types = Some((
                                    <EnvTypes as TryFrom<_>>::try_from(name_value)?,
                                    arg,
                                ));
                            }
                        }
                        syn::Meta::Path(_) | syn::Meta::List(_) => {
                            return Err(Error::invalid_arg(
                                arg,
                                "encountered invalid or unknown ink! argument",
                            ))
                        }
                    }
                }
            }
        }
        Ok(Config {
            version: version.map(|(version, _)| version).ok_or_else(|| {
                Error::missing_arg("missing ink! version config argument")
            })?,
            storage_alloc: storage_alloc.map(|(storage_alloc, _)| storage_alloc),
            as_dependency: as_dependency.map(|(as_dependency, _)| as_dependency),
            env_types: env_types.map(|(env_types, _)| env_types),
        })
    }
}

/// Convenience function to convert a [`MetaNameValue`](`syn::MetaNameValue`)
/// into a [`NestedMeta`](`syn::NestedMeta`).
fn into_nested_meta(name_value: &syn::MetaNameValue) -> syn::NestedMeta {
    syn::NestedMeta::Meta(syn::Meta::NameValue(name_value.clone()))
}

impl<'a> TryFrom<&'a syn::MetaNameValue> for EnvTypes {
    type Error = Error;

    fn try_from(name_value: &syn::MetaNameValue) -> Result<Self, Self::Error> {
        if !name_value.path.is_ident("env_types") {
            return Err(Error::invalid_arg(
                into_nested_meta(name_value),
                "invalid ink! `env_types` argument identifier",
            ))
        }
        match &name_value.lit {
            syn::Lit::Str(lit_str) => {
                let path_str = lit_str.value();
                let path = syn::parse_str::<syn::Path>(&path_str).map_err(|_| {
                    Error::invalid_arg(
                        into_nested_meta(name_value),
                        "expected path or identifier for `env_types` argument value",
                    )
                })?;
                Ok(EnvTypes { env_types: path })
            }
            _ => {
                Err(Error::invalid_arg(
                    into_nested_meta(name_value),
                    "expected a string literal for `env_types` ink! argument",
                ))
            }
        }
    }
}

impl<'a> TryFrom<&'a syn::MetaNameValue> for InkVersion {
    type Error = Error;

    fn try_from(name_value: &syn::MetaNameValue) -> Result<Self, Self::Error> {
        if !name_value.path.is_ident("version") {
            return Err(Error::invalid_arg(
                into_nested_meta(name_value),
                "invalid ink! version argument identifier",
            ))
        }
        let version_str = if let syn::Lit::Str(version_str) = &name_value.lit {
            version_str.value()
        } else {
            return Err(Error::invalid_arg(
                into_nested_meta(name_value),
                "ink! version argument expects a string",
            ))
        };
        let re = Regex::new(
            r"(?x)
            ^(?P<major>0|[1-9]\d*) # major version
            \.
            (?P<minor>0|[1-9]\d*)  # minor version
            \.
            (?P<patch>0|[1-9]\d*)  # patch version

            (?:-
                (?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)
                (?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))
            *))?

            (?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
        ",
        )
        .expect("semantic version matching regex is invalid");
        let caps = re.captures(&version_str).ok_or_else(|| {
            Error::invalid_arg(
                into_nested_meta(name_value),
                "couldn't parse version string as semantic version major.minor.path, e.g. \"0.1.0\"",
            )
        })?;
        let major = caps["major"]
            .parse::<usize>()
            .expect("major version parsing cannot fail since guaranteed by regex; qed");
        let minor = caps["minor"]
            .parse::<usize>()
            .expect("minor version parsing cannot fail since guaranteed by regex; qed");
        let patch = caps["patch"]
            .parse::<usize>()
            .expect("patch version parsing cannot fail since guaranteed by regex; qed");
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

pub enum Error {
    InvalidArg {
        invalid: syn::NestedMeta,
        reason: String,
    },
    DuplicateArg {
        fst: syn::NestedMeta,
        snd: syn::NestedMeta,
        reason: String,
    },
    MissingArg {
        reason: String,
    },
}

impl Error {
    /// Creates a new error indicating an invalid config argument.
    ///
    /// Use the reason to further specify what happened.
    pub fn invalid_arg<S>(arg: syn::NestedMeta, reason: S) -> Self
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
    pub fn duplicate_arg<S>(fst: syn::NestedMeta, snd: syn::NestedMeta, reason: S) -> Self
    where
        S: Into<String>,
    {
        Self::DuplicateArg {
            fst,
            snd,
            reason: reason.into(),
        }
    }

    /// Creates an error indicating a missing required ink! config argument.
    pub fn missing_arg<S>(reason: S) -> Self
    where
        S: Into<String>,
    {
        Self::MissingArg {
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

/// The ink! version triple given as the configuration.
pub struct InkVersion {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

/// The environmental types definition.
#[derive(Clone)]
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
