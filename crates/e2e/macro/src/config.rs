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

use darling::{
    FromMeta,
    ast::NestedMeta,
};
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Meta,
    punctuated::Punctuated,
    spanned::Spanned,
};

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
#[derive(Clone, Eq, PartialEq, Debug, darling::FromMeta)]
pub struct RuntimeOnly {
    /// The runtime type (e.g., `ink_runtime::DefaultRuntime`)
    pub runtime: syn::Path,
    /// The client type implementing the backend traits (e.g.,
    /// `ink_runtime::RuntimeClient`)
    pub client: syn::Path,
}

impl RuntimeOnly {
    pub fn runtime_path(&self) -> syn::Path {
        self.runtime.clone()
    }
    pub fn client_path(&self) -> syn::Path {
        self.client.clone()
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

    /// Parses the attribute arguments passed to `ink_e2e::test`.
    pub fn from_attr_tokens(attr: TokenStream2) -> Result<Self, syn::Error> {
        let nested_meta = NestedMeta::parse_meta_list(attr)?;
        Self::from_nested_meta(nested_meta)
    }

    /// Builds the configuration from already parsed meta items.
    pub fn from_nested_meta(nested_meta: Vec<NestedMeta>) -> Result<Self, syn::Error> {
        let normalized = normalize_runtime_meta(nested_meta)?;
        Self::from_list(&normalized).map_err(syn::Error::from)
    }
}

fn normalize_runtime_meta(
    nested_meta: Vec<NestedMeta>,
) -> Result<Vec<NestedMeta>, syn::Error> {
    let mut args = Vec::with_capacity(nested_meta.len());
    let mut runtime = None;

    for meta in nested_meta {
        if let Some(found) = RuntimeBackendArg::from_nested_meta(&meta)? {
            if runtime.replace(found).is_some() {
                return Err(syn::Error::new(
                    meta.span(),
                    "only a single `runtime` attribute is allowed",
                ));
            }
            continue;
        }
        args.push(meta);
    }

    if let Some(runtime) = runtime {
        args.push(runtime.into_backend_meta());
    }

    Ok(args)
}

struct RuntimeBackendArg {
    runtime: Option<syn::Path>,
}

impl RuntimeBackendArg {
    fn from_nested_meta(meta: &NestedMeta) -> Result<Option<Self>, syn::Error> {
        let meta = match meta {
            NestedMeta::Meta(meta) if meta.path().is_ident("runtime") => meta,
            _ => return Ok(None),
        };

        match meta {
            Meta::Path(_) => Ok(Some(Self { runtime: None })),
            Meta::List(list) => {
                let nested: Punctuated<NestedMeta, syn::Token![,]> =
                    list.parse_args_with(Punctuated::parse_terminated)?;
                if nested.len() != 1 {
                    return Err(syn::Error::new(
                        list.span(),
                        "`runtime` expects zero or one runtime type",
                    ));
                }
                match nested.first().unwrap() {
                    NestedMeta::Meta(Meta::Path(path)) => {
                        Ok(Some(Self {
                            runtime: Some(path.clone()),
                        }))
                    }
                    other => {
                        Err(syn::Error::new(
                            other.span(),
                            "`runtime` expects a runtime type path",
                        ))
                    }
                }
            }
            Meta::NameValue(name_value) => {
                Err(syn::Error::new(
                    name_value.span(),
                    "`runtime` does not support name-value pairs",
                ))
            }
        }
    }

    fn runtime(&self) -> syn::Path {
        self.runtime
            .clone()
            .unwrap_or_else(|| syn::parse_quote! { ::ink_runtime::DefaultRuntime })
    }

    fn into_backend_meta(self) -> NestedMeta {
        let runtime = self.runtime();
        syn::parse_quote! {
            backend(runtime_only(runtime = #runtime, client = ::ink_runtime::RuntimeClient))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::ast::NestedMeta;
    use quote::quote;

    fn parse_config(input: TokenStream2) -> E2EConfig {
        let nested = NestedMeta::parse_meta_list(input).unwrap();
        E2EConfig::from_nested_meta(nested).unwrap()
    }

    #[test]
    fn config_works_backend_runtime_only() {
        let input = quote! {
            environment = crate::CustomEnvironment,
            backend(runtime_only(runtime = ::ink_runtime::DefaultRuntime, client = ::ink_runtime::RuntimeClient)),
        };
        let config = parse_config(input);

        assert_eq!(
            config.environment(),
            Some(syn::parse_quote! { crate::CustomEnvironment })
        );

        assert_eq!(
            config.backend(),
            Backend::RuntimeOnly(RuntimeOnly {
                runtime: syn::parse_quote! { ::ink_runtime::DefaultRuntime },
                client: syn::parse_quote! { ::ink_runtime::RuntimeClient },
            })
        );
    }

    #[test]
    #[should_panic(expected = "Unknown field")]
    fn config_backend_runtime_only_default_not_allowed() {
        let input = quote! {
            backend(runtime_only(default)),
        };
        let config = parse_config(input);

        assert_eq!(
            config.backend(),
            Backend::RuntimeOnly(RuntimeOnly {
                runtime: syn::parse_quote! { ::ink_runtime::DefaultRuntime },
                client: syn::parse_quote! { ::ink_runtime::RuntimeClient },
            })
        );
    }

    #[test]
    fn config_works_runtime_only_with_custom_backend() {
        let input = quote! {
            backend(runtime_only(runtime = ::ink_runtime::DefaultRuntime, client = ::ink_runtime::RuntimeClient)),
        };
        let config = parse_config(input);

        assert_eq!(
            config.backend(),
            Backend::RuntimeOnly(RuntimeOnly {
                runtime: syn::parse_quote! { ::ink_runtime::DefaultRuntime },
                client: syn::parse_quote! { ::ink_runtime::RuntimeClient },
            })
        );
    }

    #[test]
    fn runtime_keyword_defaults() {
        let input = quote! { runtime };
        let config = parse_config(input);

        assert_eq!(
            config.backend(),
            Backend::RuntimeOnly(RuntimeOnly {
                runtime: syn::parse_quote! { ::ink_runtime::DefaultRuntime },
                client: syn::parse_quote! { ::ink_runtime::RuntimeClient },
            })
        );
    }

    #[test]
    fn runtime_keyword_custom_runtime() {
        let input = quote! { runtime(crate::CustomRuntime) };
        let config = parse_config(input);

        assert_eq!(
            config.backend(),
            Backend::RuntimeOnly(RuntimeOnly {
                runtime: syn::parse_quote! { crate::CustomRuntime },
                client: syn::parse_quote! { ::ink_runtime::RuntimeClient },
            })
        );
    }

    #[test]
    fn config_works_backend_node() {
        let input = quote! {
            backend(node),
        };
        let config = parse_config(input);

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
    #[should_panic(expected = "Unknown field")]
    fn config_backend_node_auto_not_allowed() {
        let input = quote! {
            backend(node(auto)),
        };
        let config = parse_config(input);

        assert_eq!(config.backend(), Backend::Node(Node::Auto));
    }

    #[test]
    fn config_works_backend_node_url() {
        let input = quote! {
            backend(node(url = "ws://0.0.0.0:9999")),
        };
        let config = parse_config(input);

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
        let config = parse_config(input);

        assert_eq!(config.replace_test_attr(), Some("#[quickcheck]".to_owned()));
    }
}
