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

/// The type of the architecture that should be used to run test.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, darling::FromMeta)]
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

/// The End-to-End test configuration.
#[derive(Debug, Default, PartialEq, Eq, darling::FromMeta)]
pub struct E2EConfig {
    /// Additional contracts that have to be built before executing the test.
    #[darling(default)]
    additional_contracts: String,
    /// The [`Environment`](https://docs.rs/ink_env/4.1.0/ink_env/trait.Environment.html) to use
    /// during test execution.
    ///
    /// If no `Environment` is specified, the
    /// [`DefaultEnvironment`](https://docs.rs/ink_env/4.1.0/ink_env/enum.DefaultEnvironment.html)
    /// will be used.
    #[darling(default)]
    environment: Option<syn::Path>,
    /// The type of the architecture that should be used to run test.
    #[darling(default)]
    backend: Backend,
    /// The runtime to use for the runtime_only test.
    #[cfg(any(test, feature = "drink"))]
    #[darling(default)]
    runtime: Option<syn::Path>,
}

impl E2EConfig {
    /// Returns a vector of additional contracts that have to be built
    /// and imported before executing the test.
    pub fn additional_contracts(&self) -> Vec<String> {
        self.additional_contracts
            .split(' ')
            .map(String::from)
            .collect()
    }

    /// Custom environment for the contracts, if specified.
    pub fn environment(&self) -> Option<syn::Path> {
        self.environment.clone()
    }

    /// The type of the architecture that should be used to run test.
    pub fn backend(&self) -> Backend {
        self.backend
    }

    /// The runtime to use for the runtime_only test.
    #[cfg(any(test, feature = "drink"))]
    pub fn runtime(&self) -> Option<syn::Path> {
        self.runtime.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::{
        ast::NestedMeta,
        FromMeta,
    };
    use proc_macro2::TokenStream;
    use quote::quote;

    /// Asserts that the given input configuration attribute argument are converted
    /// into the expected ink! configuration or yields the expected error message.
    fn assert_try_from(input: TokenStream, expected: Result<E2EConfig, &'static str>) {
        assert_eq!(
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input.into()).unwrap())
                .map_err(|err| err.to_string()),
            expected.map_err(ToString::to_string),
        );
    }

    #[test]
    fn empty_config_works() {
        assert_try_from(quote! {}, Ok(E2EConfig::default()))
    }

    #[test]
    fn unknown_arg_fails() {
        assert_try_from(
            quote! { unknown = argument },
            Err("Unknown field: `unknown`"),
        );
    }

    #[test]
    fn duplicate_additional_contracts_fails() {
        assert_try_from(
            quote! {
                additional_contracts = "adder/Cargo.toml",
                additional_contracts = "adder/Cargo.toml",
            },
            Err("Duplicate field `additional_contracts`"),
        );
    }

    #[test]
    fn duplicate_environment_fails() {
        assert_try_from(
            quote! {
                environment = crate::CustomEnvironment,
                environment = crate::CustomEnvironment,
            },
            Err("Duplicate field `environment`"),
        );
    }

    #[test]
    fn specifying_environment_works() {
        assert_try_from(
            quote! {
                environment = crate::CustomEnvironment,
            },
            Ok(E2EConfig {
                environment: Some(syn::parse_quote! { crate::CustomEnvironment }),
                ..Default::default()
            }),
        );

        assert_try_from(
            quote! {
                environment = "crate::CustomEnvironment",
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
            quote! { backend = full },
            Err("Unexpected literal type `path` at backend"),
        );
    }

    #[test]
    fn duplicate_backend_fails() {
        assert_try_from(
            quote! {
                backend = "full",
                backend = "runtime_only",
            },
            Err("Duplicate field `backend`"),
        );
    }

    #[test]
    fn specifying_backend_works() {
        assert_try_from(
            quote! { backend = "runtime_only" },
            Ok(E2EConfig {
                backend: Backend::RuntimeOnly,
                ..Default::default()
            }),
        );
    }

    #[test]
    fn duplicate_runtime_fails() {
        assert_try_from(
            quote! {
                runtime = ::drink::MinimalRuntime,
                runtime = ::drink::MaximalRuntime,
            },
            Err("Duplicate field `runtime`"),
        );
    }

    #[test]
    fn specifying_runtime_works() {
        assert_try_from(
            quote! {
                backend = "runtime_only",
                runtime = ::drink::MinimalRuntime
            },
            Ok(E2EConfig {
                backend: Backend::RuntimeOnly,
                runtime: Some(syn::parse_quote! { ::drink::MinimalRuntime }),
                ..Default::default()
            }),
        );

        assert_try_from(
            quote! {
                backend = "runtime_only",
                runtime = "::drink::MinimalRuntime"
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
            quote! {
                additional_contracts = "adder/Cargo.toml flipper/Cargo.toml",
                environment = crate::CustomEnvironment,
                backend = "runtime_only",
                runtime = ::drink::MinimalRuntime,
            },
            Ok(E2EConfig {
                additional_contracts: "adder/Cargo.toml flipper/Cargo.toml".to_string(),
                environment: Some(syn::parse_quote! { crate::CustomEnvironment }),
                backend: Backend::RuntimeOnly,
                runtime: Some(syn::parse_quote! { ::drink::MinimalRuntime }),
            }),
        );
    }
}
