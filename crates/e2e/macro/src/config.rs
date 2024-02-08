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
#[derive(Clone, Eq, PartialEq, Debug, darling::FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum Backend {
    /// The standard approach with running dedicated single-node blockchain in a
    /// background process.
    Node(Node),

    /// The lightweight approach skipping node layer.
    ///
    /// This runs a runtime emulator within `TestExternalities` (using drink! library) in
    /// the same process as the test.
    #[cfg(any(test, feature = "drink"))]
    RuntimeOnly(RuntimeOnly),
}

impl Default for Backend {
    fn default() -> Self {
        Backend::Node(Node::Auto)
    }
}

/// Configure whether to automatically spawn a node instance for the test or to use
/// an already running node at the supplied URL.
#[derive(Clone, Eq, PartialEq, Debug, darling::FromMeta)]
pub enum Node {
    /// A fresh node instance will be spawned for the lifetime of the test.
    #[darling(word)]
    Auto,
    /// The test will run against an already running node at the supplied URL.
    Url(String),
}

impl Node {
    /// The URL to the running node, default value can be overridden with
    /// `CONTRACTS_NODE_URL`.
    ///
    /// Returns `None` if [`Self::Auto`] and `CONTRACTS_NODE_URL` not specified.
    pub fn url(&self) -> Option<String> {
        std::env::var("CONTRACTS_NODE_URL").ok().or_else(|| {
            match self {
                Node::Auto => None,
                Node::Url(url) => Some(url.clone()),
            }
        })
    }
}

/// The runtime emulator that should be used within `TestExternalities` (using drink!
/// library).
#[cfg(any(test, feature = "drink"))]
#[derive(Clone, Eq, PartialEq, Debug, darling::FromMeta)]
pub enum RuntimeOnly {
    #[darling(word)]
    Default,
    Runtime(syn::Path),
}

#[cfg(any(test, feature = "drink"))]
impl From<RuntimeOnly> for syn::Path {
    fn from(value: RuntimeOnly) -> Self {
        match value {
            RuntimeOnly::Default => syn::parse_quote! { ::ink_e2e::MinimalRuntime },
            RuntimeOnly::Runtime(path) => path,
        }
    }
}

/// The End-to-End test configuration.
#[derive(Debug, Default, PartialEq, Eq, darling::FromMeta)]
pub struct E2EConfig {
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
}

impl E2EConfig {
    /// Custom environment for the contracts, if specified.
    pub fn environment(&self) -> Option<syn::Path> {
        self.environment.clone()
    }

    /// The type of the architecture that should be used to run test.
    pub fn backend(&self) -> Backend {
        self.backend.clone()
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
    fn config_works_backend_runtime_only() {
        let input = quote! {
            environment = crate::CustomEnvironment,
            backend(runtime_only),
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        assert_eq!(
            config.environment(),
            Some(syn::parse_quote! { crate::CustomEnvironment })
        );

        assert_eq!(config.backend(), Backend::RuntimeOnly(RuntimeOnly::Default));
    }

    #[test]
    fn config_works_runtime_only_with_custom_backend() {
        let input = quote! {
            backend(runtime_only(runtime = ::ink_e2e::MinimalRuntime)),
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        assert_eq!(
            config.backend(),
            Backend::RuntimeOnly(RuntimeOnly::Runtime(
                syn::parse_quote! { ::ink_e2e::MinimalRuntime }
            ))
        );
    }

    #[test]
    fn config_works_backend_node_default_auto() {
        let input = quote! {
            backend(node),
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        assert_eq!(config.backend(), Backend::Node(Node::Auto));
    }

    #[test]
    fn config_works_backend_node_auto() {
        let input = quote! {
            backend(node(auto)),
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        match config.backend() {
            Backend::Node(node_config) => {
                assert_eq!(node_config, Node::Auto);

                temp_env::with_vars([("CONTRACTS_NODE_URL", None::<&str>)], || {
                    assert_eq!(node_config.url(), None);
                });

                temp_env::with_vars(
                    [("CONTRACTS_NODE_URL", Some("ws://127.0.0.1:9000"))],
                    || {
                        assert_eq!(
                            node_config.url(),
                            Some(String::from("ws://127.0.0.1:9000"))
                        );
                    },
                );
            }
            _ => panic!("Expected Backend::Node"),
        }
    }

    #[test]
    fn config_works_backend_node_url() {
        let input = quote! {
            backend(node(url = "ws://0.0.0.0:9999")),
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        match config.backend() {
            Backend::Node(node_config) => {
                assert_eq!(node_config, Node::Url("ws://0.0.0.0:9999".to_owned()));

                temp_env::with_vars([("CONTRACTS_NODE_URL", None::<&str>)], || {
                    assert_eq!(node_config.url(), Some("ws://0.0.0.0:9999".to_owned()));
                });

                temp_env::with_vars(
                    [("CONTRACTS_NODE_URL", Some("ws://127.0.0.1:9000"))],
                    || {
                        assert_eq!(
                            node_config.url(),
                            Some(String::from("ws://127.0.0.1:9000"))
                        );
                    },
                );
            }
            _ => panic!("Expected Backend::Node"),
        }
    }
}
