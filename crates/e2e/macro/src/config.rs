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
#[derive(Clone, Eq, PartialEq, Debug, Default, darling::FromMeta)]
#[darling(rename_all = "snake_case")]
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
    RuntimeOnly { runtime: Option<syn::Path> },
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
    /// The URL to the running node. If not set then a default node instance will be
    /// spawned per test.
    node_url: Option<String>,
}

impl E2EConfig {
    /// Returns a vector of additional contracts that have to be built
    /// and imported before executing the test.
    pub fn additional_contracts(&self) -> Vec<String> {
        self.additional_contracts
            .split(' ')
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_owned())
                }
            })
            .collect()
    }

    /// Custom environment for the contracts, if specified.
    pub fn environment(&self) -> Option<syn::Path> {
        self.environment.clone()
    }

    /// The type of the architecture that should be used to run test.
    pub fn backend(&self) -> Backend {
        self.backend.clone()
    }

    /// The URL to the running node. If not set then a default node instance will be
    /// spawned per test.
    pub fn node_url(&self) -> Option<String> {
        self.node_url.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::{
        ast::NestedMeta,
        FromMeta,
    };
    use quote::quote;

    #[test]
    fn config_works() {
        let input = quote! {
            additional_contracts = "adder/Cargo.toml flipper/Cargo.toml",
            environment = crate::CustomEnvironment,
            backend(runtime_only()),
            node_url = "ws://127.0.0.1:8000"
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        assert_eq!(
            config.additional_contracts(),
            vec!["adder/Cargo.toml", "flipper/Cargo.toml"]
        );
        assert_eq!(
            config.environment(),
            Some(syn::parse_quote! { crate::CustomEnvironment })
        );

        assert_eq!(config.backend(), Backend::RuntimeOnly { runtime: None });
        assert_eq!(config.node_url(), Some(String::from("ws://127.0.0.1:8000")))
    }

    #[test]
    fn config_works_with_custom_backend() {
        let input = quote! {
            backend(runtime_only(runtime = ::ink_e2e::MinimalRuntime)),
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        assert_eq!(
            config.backend(),
            Backend::RuntimeOnly {
                runtime: Some(syn::parse_quote! { ::ink_e2e::MinimalRuntime })
            }
        );
        assert_eq!(config.node_url(), None)
    }
}
