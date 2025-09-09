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

use crate::{
    ast,
    utils::{
        WhitelistedAttributes,
        duplicate_config_err,
    },
};

/// The ink! configuration.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TraitDefinitionConfig {
    /// Captures the optional custom namespace for the ink! trait definition.
    ///
    /// # Note
    ///
    /// The namespace configuration parameter is used to influence the generated
    /// selectors of the ink! trait messages. This is useful to disambiguate
    /// ink! trait definitions with equal names.
    namespace: Option<syn::LitStr>,
    /// The set of attributes that can be passed to call builder and forwarder in the
    /// codegen.
    whitelisted_attributes: WhitelistedAttributes,
}

impl TraitDefinitionConfig {
    /// Sets the namespace of the ink! trait definition configuration.
    ///
    /// # Note
    ///
    /// This is a test-only API.
    #[cfg(test)]
    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace =
            Some(syn::LitStr::new(namespace, proc_macro2::Span::call_site()));
        self
    }
}

impl TryFrom<ast::AttributeArgs> for TraitDefinitionConfig {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut namespace: Option<(syn::LitStr, ast::MetaNameValue)> = None;
        let mut whitelisted_attributes = WhitelistedAttributes::default();
        for arg in args.into_iter() {
            if arg.name().is_ident("namespace") {
                if let Some((_, meta_name_value)) = namespace {
                    return Err(duplicate_config_err(
                        meta_name_value,
                        arg,
                        "namespace",
                        "trait definition",
                    ));
                }
                let namespace_info = arg
                    .name_value()
                    .zip(arg.value().and_then(ast::MetaValue::as_lit_string));
                if let Some((name_value, lit_str)) = namespace_info {
                    if syn::parse_str::<syn::Ident>(&lit_str.value()).is_err() {
                        return Err(format_err_spanned!(
                            lit_str,
                            "encountered invalid Rust identifier for the ink! namespace configuration parameter"
                        ));
                    }
                    namespace = Some((lit_str.clone(), name_value.clone()))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string literal value for `namespace` ink! trait definition configuration argument",
                    ));
                }
            } else if arg.name().is_ident("keep_attr") {
                if let Some(name_value) = arg.name_value() {
                    whitelisted_attributes.parse_arg_value(name_value)?;
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string literal value for `keep_attr` ink! configuration argument",
                    ));
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! trait definition configuration argument",
                ));
            }
        }
        Ok(TraitDefinitionConfig {
            namespace: namespace.map(|(value, _)| value),
            whitelisted_attributes,
        })
    }
}

impl TraitDefinitionConfig {
    /// Returns the namespace configuration argument if any as string.
    pub fn namespace(&self) -> Option<&syn::LitStr> {
        self.namespace.as_ref()
    }

    /// Returns the set of attributes that can be passed to call builder and
    /// forwarder in the codegen.
    pub fn whitelisted_attributes(&self) -> &WhitelistedAttributes {
        &self.whitelisted_attributes
    }
}
