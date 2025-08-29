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

/// The type of the architecture that should be used to run test.
#[derive(Clone, Eq, PartialEq, Debug, darling::FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum Backend {
    /// The standard approach with running dedicated single-node blockchain in a
    /// background process.
    Node(Node),

    /// The lightweight approach skipping node layer.
    ///
    /// This runs a runtime emulator within `TestExternalities`
    /// the same process as the test.
    #[cfg(any(test, feature = "sandbox"))]
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
    #[darling(skip)]
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
        let url = std::env::var("CONTRACTS_NODE_URL").ok().or_else(|| {
            match self {
                Node::Auto => None,
                Node::Url(url) => Some(url.clone()),
            }
        });
        tracing::debug!("[E2E] Using node url {:?}", url);
        url
    }
}

/// The runtime emulator that should be used within `TestExternalities`
#[cfg(any(test, feature = "sandbox"))]
#[derive(Clone, Eq, PartialEq, Debug, darling::FromMeta)]
pub enum RuntimeOnly {
    #[darling(word)]
    #[darling(skip)]
    Default,
    Sandbox(syn::Path),
}

#[cfg(any(test, feature = "sandbox"))]
impl From<RuntimeOnly> for syn::Path {
    fn from(value: RuntimeOnly) -> Self {
        match value {
            RuntimeOnly::Default => syn::parse_quote! { ::ink_e2e::DefaultSandbox },
            RuntimeOnly::Sandbox(path) => path,
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
    /// Features that are enabled in the contract during the build process.
    /// todo add tests below in this file
    #[darling(default)]
    features: Vec<syn::LitStr>,
    /// A replacement attribute for `#[test]`. Instead of `#[test]` the E2E code
    /// generation will output this attribute.
    ///
    /// This can be used to supply e.g. `#[quicktest]`, thus transforming the
    /// test into a fuzzing E2E test.
    #[darling(default)]
    replace_test_attr: Option<String>,
}

impl E2EConfig {
    /// Custom environment for the contracts, if specified.
    pub fn environment(&self) -> Option<syn::Path> {
        self.environment.clone()
    }

    /// Features for the contract build.
    pub fn features(&self) -> Vec<String> {
        self.features.iter().map(|ls| ls.value()).collect()
    }

    /// The type of the architecture that should be used to run test.
    pub fn backend(&self) -> Backend {
        self.backend.clone()
    }

    /// A custom attribute which the code generation will output instead
    /// of `#[test]`.
    pub fn replace_test_attr(&self) -> Option<String> {
        self.replace_test_attr.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::{
        FromMeta,
        ast::NestedMeta,
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
    #[should_panic(expected = "ErrorUnknownField")]
    fn config_backend_runtime_only_default_not_allowed() {
        let input = quote! {
            backend(runtime_only(default)),
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        assert_eq!(config.backend(), Backend::RuntimeOnly(RuntimeOnly::Default));
    }

    #[test]
    fn config_works_runtime_only_with_custom_backend() {
        let input = quote! {
            backend(runtime_only(sandbox = ::ink_e2e::DefaultSandbox)),
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        assert_eq!(
            config.backend(),
            Backend::RuntimeOnly(RuntimeOnly::Sandbox(
                syn::parse_quote! { ::ink_e2e::DefaultSandbox }
            ))
        );
    }

    #[test]
    fn config_works_backend_node() {
        let input = quote! {
            backend(node),
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        assert_eq!(config.backend(), Backend::Node(Node::Auto));

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
    #[should_panic(expected = "ErrorUnknownField")]
    fn config_backend_node_auto_not_allowed() {
        let input = quote! {
            backend(node(auto)),
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        assert_eq!(config.backend(), Backend::Node(Node::Auto));
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

    #[test]
    fn config_works_test_attr_replacement() {
        let input = quote! {
            replace_test_attr = "#[quickcheck]"
        };
        let config =
            E2EConfig::from_list(&NestedMeta::parse_meta_list(input).unwrap()).unwrap();

        assert_eq!(config.replace_test_attr(), Some("#[quickcheck]".to_owned()));
    }
}
