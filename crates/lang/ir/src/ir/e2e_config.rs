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
    utils::{
        duplicate_config_err,
        WhitelistedAttributes,
    },
};

/// The End-to-End test configuration.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct E2EConfig {
    /// The path where the node writes its log.
    node_log: Option<syn::LitStr>,
    /// The WebSocket URL where to connect with the node.
    ws_url: Option<syn::LitStr>,
    /// Denotes if the contract should be build before executing the test.
    skip_build: Option<syn::LitBool>,
    /// The set of attributes that can be passed to call builder in the codegen.
    whitelisted_attributes: WhitelistedAttributes,
}

impl TryFrom<ast::AttributeArgs> for E2EConfig {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut node_log: Option<(syn::LitStr, ast::MetaNameValue)> = None;
        let mut ws_url: Option<(syn::LitStr, ast::MetaNameValue)> = None;
        let mut skip_build: Option<(syn::LitBool, ast::MetaNameValue)> = None;
        let mut whitelisted_attributes = WhitelistedAttributes::default();

        for arg in args.into_iter() {
            if arg.name.is_ident("node_log") {
                if let Some((_, ast)) = node_log {
                    return Err(duplicate_config_err(ast, arg, "node_log"))
                }
                if let ast::PathOrLit::Lit(syn::Lit::Str(lit_str)) = &arg.value {
                    node_log = Some((lit_str.clone(), arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a path for `node_log` ink! e2e test configuration argument",
                    ))
                }
            } else if arg.name.is_ident("ws_url") {
                if let Some((_, ast)) = ws_url {
                    return Err(duplicate_config_err(ast, arg, "ws_url"))
                }
                if let ast::PathOrLit::Lit(syn::Lit::Str(lit_str)) = &arg.value {
                    ws_url = Some((lit_str.clone(), arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string literal for `ws_url` ink! e2e test configuration argument",
                    ))
                }
            } else if arg.name.is_ident("skip_build") {
                if let Some((_, ast)) = skip_build {
                    return Err(duplicate_config_err(ast, arg, "skip_build"))
                }
                if let ast::PathOrLit::Lit(syn::Lit::Bool(lit_bool)) = &arg.value {
                    skip_build = Some((lit_bool.clone(), arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a bool literal for `skip_build` ink! e2e test configuration argument",
                    ))
                }
            } else if arg.name.is_ident("keep_attr") {
                whitelisted_attributes.parse_arg_value(&arg)?;
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! configuration argument",
                ))
            }
        }
        Ok(E2EConfig {
            node_log: node_log.map(|(value, _)| value),
            ws_url: ws_url.map(|(value, _)| value),
            skip_build: skip_build.map(|(value, _)| value),
            whitelisted_attributes,
        })
    }
}

impl E2EConfig {
    /// Returns the path to the node log if specified.
    /// Otherwise returns the default path `/tmp/contracts-node.log`.
    pub fn node_log(&self) -> syn::LitStr {
        let default_node_log =
            syn::LitStr::new("/tmp/contracts-node.log", proc_macro2::Span::call_site());
        self.node_log.clone().unwrap_or(default_node_log)
    }

    /// Returns the WebSocket URL where to connect to the RPC endpoint
    /// of the node, if specified. Otherwise returns the default URL
    /// `ws://localhost:9944`.
    pub fn ws_url(&self) -> syn::LitStr {
        let default_ws_url =
            syn::LitStr::new("ws://localhost:9944", proc_macro2::Span::call_site());
        self.ws_url.clone().unwrap_or(default_ws_url)
    }

    /// Returns `true` if `skip_build = true` was configured.
    /// Otherwise returns `false`.
    pub fn skip_build(&self) -> syn::LitBool {
        let default_skip_build = syn::LitBool::new(false, proc_macro2::Span::call_site());
        self.skip_build.clone().unwrap_or(default_skip_build)
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
    fn duplicate_args_fails() {
        assert_try_from(
            syn::parse_quote! {
                skip_build = true,
                skip_build = true,
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
            Ok(E2EConfig {
                node_log: None,
                ws_url: None,
                whitelisted_attributes: attrs,
                skip_build: None,
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
