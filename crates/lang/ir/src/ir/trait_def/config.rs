// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
    error::ExtError as _,
};
use core::convert::TryFrom;
use syn::spanned::Spanned;

/// The ink! configuration.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TraitDefinitionConfig {
    /// Captures the optional custom namespace for the ink! trait definition.
    ///
    /// # Note
    ///
    /// The namespace config parameter is used to influence the generated
    /// selectors of the ink! trait messages. This is useful to disambiguate
    /// ink! trait definitions with equal names.
    namespace: Option<syn::LitStr>,
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

/// Return an error to notify about duplicate ink! trait definition configuration arguments.
fn duplicate_config_err<F, S>(fst: F, snd: S, name: &str) -> syn::Error
where
    F: Spanned,
    S: Spanned,
{
    format_err!(
        snd.span(),
        "encountered duplicate ink! trait definition `{}` configuration argument",
        name,
    )
    .into_combine(format_err!(
        fst.span(),
        "first `{}` configuration argument here",
        name
    ))
}

impl TryFrom<ast::AttributeArgs> for TraitDefinitionConfig {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut namespace: Option<(syn::LitStr, ast::MetaNameValue)> = None;
        for arg in args.into_iter() {
            if arg.name.is_ident("namespace") {
                if let Some((_, meta_name_value)) = namespace {
                    return Err(duplicate_config_err(meta_name_value, arg, "namespace"))
                }
                if let ast::PathOrLit::Lit(syn::Lit::Str(lit_str)) = &arg.value {
                    if syn::parse_str::<syn::Ident>(&lit_str.value()).is_err() {
                        return Err(format_err_spanned!(
                            lit_str,
                            "encountered invalid Rust identifier for the ink! namespace configuration parameter"
                        ))
                    }
                    namespace = Some((lit_str.clone(), arg))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string literal for `namespace` ink! trait definition configuration argument",
                    ))
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! trait definition configuration argument",
                ))
            }
        }
        Ok(TraitDefinitionConfig {
            namespace: namespace.map(|(value, _)| value),
        })
    }
}

impl TraitDefinitionConfig {
    /// Returns the namespace config argument if any as string.
    pub fn namespace(&self) -> Option<&syn::LitStr> {
        self.namespace.as_ref()
    }
}
