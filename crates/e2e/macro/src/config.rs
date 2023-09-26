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
    /// The runtime to use for the runtime only test.
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

    #[test]
    fn config_works() {
        let input = quote! {
            additional_contracts = "adder/Cargo.toml flipper/Cargo.toml",
            environment = crate::CustomEnvironment,
            backend = "runtime_only",
            runtime = ::drink::MinimalRuntime,
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input.into()).unwrap())
                .unwrap();

        assert_eq!(
            config.additional_contracts(),
            vec!["adder/Cargo.toml", "flipper/Cargo.toml"]
        );
        assert_eq!(
            config.environment(),
            Some(syn::parse_quote! { crate::CustomEnvironment })
        );
        assert_eq!(config.backend(), Backend::RuntimeOnly);
        assert_eq!(
            config.runtime(),
            Some(syn::parse_quote! { ::drink::MinimalRuntime })
        );
    }
}
