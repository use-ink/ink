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

use ink_ir::{
    ast,
    format_err_spanned,
    utils::duplicate_config_err,
};

/// The type of the architecture that should be used to run test.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum Backend {
    /// The standard approach with running dedicated single-node blockchain in a
    /// background process.
    #[default]
    Full,
    /// The lightweight approach skipping node layer.
    ///
    /// This runs a runtime emulator within `TestExternalities` (using drink! library) in
    /// the same process as the test.
    #[cfg(any(test, feature = "drink"))]
    RuntimeOnly,
}

impl TryFrom<syn::LitStr> for Backend {
    type Error = syn::Error;

    fn try_from(value: syn::LitStr) -> Result<Self, Self::Error> {
        match value.value().as_str() {
            "full" => Ok(Self::Full),
            #[cfg(any(test, feature = "drink"))]
            "runtime_only" | "runtime-only" => Ok(Self::RuntimeOnly),
            #[cfg(not(any(test, feature = "drink")))]
            "runtime_only" | "runtime-only" => Err(
                format_err_spanned!(
                    value,
                    "the `runtime-only` backend is not available because the `drink` feature is not enabled",
                )
            ),
            _ => {
                Err(format_err_spanned!(
                    value,
                    "unknown backend `{}` for ink! E2E test configuration argument",
                    value.value()
                ))
            }
        }
    }
}

/// The End-to-End test configuration.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct E2EConfig {
    /// Additional contracts that have to be built before executing the test.
    additional_contracts: Vec<String>,
    /// The [`Environment`](https://docs.rs/ink_env/4.1.0/ink_env/trait.Environment.html) to use
    /// during test execution.
    ///
    /// If no `Environment` is specified, the
    /// [`DefaultEnvironment`](https://docs.rs/ink_env/4.1.0/ink_env/enum.DefaultEnvironment.html)
    /// will be used.
    environment: Option<syn::Path>,
    /// The type of the architecture that should be used to run test.
    backend: Backend,
    /// The runtime to use for the runtime-only test.
    #[cfg(any(test, feature = "drink"))]
    runtime: Option<syn::Path>,
}

impl TryFrom<ast::AttributeArgs> for E2EConfig {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut additional_contracts: Option<(syn::LitStr, ast::MetaNameValue)> = None;
        let mut environment: Option<(syn::Path, ast::MetaNameValue)> = None;
        let mut backend: Option<(syn::LitStr, ast::MetaNameValue)> = None;
        #[cfg(any(test, feature = "drink"))]
        let mut runtime: Option<(syn::Path, ast::MetaNameValue)> = None;

        for arg in args.into_iter() {
            if arg.name.is_ident("additional_contracts") {
                if let Some((_, ast)) = additional_contracts {
                    return Err(duplicate_config_err(
                        ast,
                        arg,
                        "additional_contracts",
                        "E2E test",
                    ))
                }
                if let ast::MetaValue::Lit(syn::Lit::Str(lit_str)) = &arg.value {
                    additional_contracts = Some((lit_str.clone(), arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string literal for `additional_contracts` ink! E2E test configuration argument",
                    ));
                }
            } else if arg.name.is_ident("environment") {
                if let Some((_, ast)) = environment {
                    return Err(duplicate_config_err(ast, arg, "environment", "E2E test"))
                }
                if let ast::MetaValue::Path(path) = &arg.value {
                    environment = Some((path.clone(), arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a path for `environment` ink! E2E test configuration argument",
                    ));
                }
            } else if arg.name.is_ident("backend") {
                if let Some((_, ast)) = backend {
                    return Err(duplicate_config_err(ast, arg, "backend", "E2E test"))
                }
                if let ast::MetaValue::Lit(syn::Lit::Str(lit_str)) = &arg.value {
                    backend = Some((lit_str.clone(), arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string literal for `backend` ink! E2E test configuration argument",
                    ));
                }
            } else if arg.name.is_ident("runtime") {
                #[cfg(any(test, feature = "drink"))]
                {
                    if let Some((_, ast)) = runtime {
                        return Err(duplicate_config_err(ast, arg, "runtime", "E2E test"))
                    }
                    if let ast::MetaValue::Path(path) = &arg.value {
                        runtime = Some((path.clone(), arg))
                    } else {
                        return Err(format_err_spanned!(
                        arg,
                        "expected a path for `runtime` ink! E2E test configuration argument",
                    ));
                    }
                }
                #[cfg(not(any(test, feature = "drink")))]
                {
                    return Err(format_err_spanned!(
                        arg,
                        "the `runtime` ink! E2E test configuration argument is not available because the `drink` feature is not enabled",
                    ));
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
        let environment = environment.map(|(path, _)| path);
        let backend = backend
            .map(|(b, _)| Backend::try_from(b))
            .transpose()?
            .unwrap_or_default();

        #[cfg(any(test, feature = "drink"))]
        {
            if backend == Backend::Full && runtime.is_some() {
                return Err(format_err_spanned!(
                    runtime.unwrap().1,
                    "ink! E2E test `runtime` configuration argument is available for `runtime-only` backend only",
                ));
            }
        }

        Ok(E2EConfig {
            additional_contracts,
            environment,
            backend,
            #[cfg(any(test, feature = "drink"))]
            runtime: runtime.map(|(path, _)| path),
        })
    }
}

impl E2EConfig {
    /// Returns a vector of additional contracts that have to be built
    /// and imported before executing the test.
    pub fn additional_contracts(&self) -> Vec<String> {
        self.additional_contracts.clone()
    }

    /// Custom environment for the contracts, if specified.
    pub fn environment(&self) -> Option<syn::Path> {
        self.environment.clone()
    }

    /// The type of the architecture that should be used to run test.
    pub fn backend(&self) -> Backend {
        self.backend
    }

    /// The runtime to use for the runtime-only test.
    #[cfg(any(test, feature = "drink"))]
    pub fn runtime(&self) -> Option<syn::Path> {
        self.runtime.clone()
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
    fn duplicate_additional_contracts_fails() {
        assert_try_from(
            syn::parse_quote! {
                additional_contracts = "adder/Cargo.toml",
                additional_contracts = "adder/Cargo.toml",
            },
            Err(
                "encountered duplicate ink! E2E test `additional_contracts` configuration argument",
            ),
        );
    }

    #[test]
    fn duplicate_environment_fails() {
        assert_try_from(
            syn::parse_quote! {
                environment = crate::CustomEnvironment,
                environment = crate::CustomEnvironment,
            },
            Err(
                "encountered duplicate ink! E2E test `environment` configuration argument",
            ),
        );
    }

    #[test]
    fn environment_as_literal_fails() {
        assert_try_from(
            syn::parse_quote! {
                environment = "crate::CustomEnvironment",
            },
            Err("expected a path for `environment` ink! E2E test configuration argument"),
        );
    }

    #[test]
    fn specifying_environment_works() {
        assert_try_from(
            syn::parse_quote! {
                environment = crate::CustomEnvironment,
            },
            Ok(E2EConfig {
                environment: Some(syn::parse_quote! { crate::CustomEnvironment }),
                ..Default::default()
            }),
        );
    }

    #[test]
    fn backend_must_be_literal() {
        assert_try_from(
            syn::parse_quote! { backend = full },
            Err("expected a string literal for `backend` ink! E2E test configuration argument"),
        );
    }

    #[test]
    fn duplicate_backend_fails() {
        assert_try_from(
            syn::parse_quote! {
                backend = "full",
                backend = "runtime-only",
            },
            Err("encountered duplicate ink! E2E test `backend` configuration argument"),
        );
    }

    #[test]
    fn specifying_backend_works() {
        assert_try_from(
            syn::parse_quote! { backend = "runtime-only" },
            Ok(E2EConfig {
                backend: Backend::RuntimeOnly,
                ..Default::default()
            }),
        );
    }

    #[test]
    fn runtime_must_be_path() {
        assert_try_from(
            syn::parse_quote! { runtime = "MinimalRuntime" },
            Err("expected a path for `runtime` ink! E2E test configuration argument"),
        );
    }

    #[test]
    fn duplicate_runtime_fails() {
        assert_try_from(
            syn::parse_quote! {
                runtime = ::drink::MinimalRuntime,
                runtime = ::drink::MaximalRuntime,
            },
            Err("encountered duplicate ink! E2E test `runtime` configuration argument"),
        );
    }

    #[test]
    fn runtime_is_for_runtime_only_backend_only() {
        assert_try_from(
            syn::parse_quote! {
                backend = "full",
                runtime = ::drink::MinimalRuntime
            },
            Err("ink! E2E test `runtime` configuration argument is available for `runtime-only` backend only"),
        );
    }

    #[test]
    fn specifying_runtime_works() {
        assert_try_from(
            syn::parse_quote! {
                backend = "runtime-only",
                runtime = ::drink::MinimalRuntime
            },
            Ok(E2EConfig {
                backend: Backend::RuntimeOnly,
                runtime: Some(syn::parse_quote! { ::drink::MinimalRuntime }),
                ..Default::default()
            }),
        );
    }

    #[test]
    fn full_config_works() {
        assert_try_from(
            syn::parse_quote! {
                additional_contracts = "adder/Cargo.toml flipper/Cargo.toml",
                environment = crate::CustomEnvironment,
                backend = "runtime-only",
                runtime = ::drink::MinimalRuntime,
            },
            Ok(E2EConfig {
                additional_contracts: vec![
                    "adder/Cargo.toml".into(),
                    "flipper/Cargo.toml".into(),
                ],
                environment: Some(syn::parse_quote! { crate::CustomEnvironment }),
                backend: Backend::RuntimeOnly,
                runtime: Some(syn::parse_quote! { ::drink::MinimalRuntime }),
            }),
        );
    }
}
