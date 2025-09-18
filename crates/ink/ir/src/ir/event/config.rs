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

use syn::spanned::Spanned;

use super::SignatureTopic;
use crate::{
    ast,
    utils::{
        duplicate_config_err,
        extract_name_override,
    },
};

/// The configuration arguments to the `#[ink::event(..)]` attribute macro.
#[derive(Debug, PartialEq, Eq)]
pub struct EventConfig {
    /// If set to `false`, a signature topic is generated and emitted for this event.
    /// If set to `true`, **no** signature topic is generated or emitted for this event.,
    /// This is the default value.
    anonymous: bool,
    /// Manually specified signature topic hash.
    signature_topic: Option<SignatureTopic>,
    /// An optional event name override.
    ///
    /// # Note
    ///
    /// - Useful for defining overloaded interfaces.
    /// - If provided, the name must be a valid "identifier-like" string.
    name: Option<String>,
}

impl EventConfig {
    /// Parse a new [`EventConfig`] from a list of attribute meta items.
    pub fn parse<I>(args: I) -> Result<Self, syn::Error>
    where
        I: Iterator<Item = ast::Meta>,
    {
        let mut anonymous: Option<syn::Path> = None;
        let mut signature_topic: Option<syn::LitStr> = None;
        let mut name: Option<syn::LitStr> = None;
        for arg in args {
            if arg.name().is_ident("anonymous") {
                if let Some(lit_bool) = anonymous {
                    return Err(duplicate_config_err(lit_bool, arg, "anonymous", "event"));
                }
                if let ast::Meta::Path(path) = arg {
                    anonymous = Some(path)
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "encountered an unexpected value for `anonymous` ink! event item configuration argument. \
                        Did you mean #[ink::event(anonymous)] ?",
                    ));
                }
            } else if arg.name().is_ident("signature_topic") {
                if anonymous.is_some() {
                    return Err(format_err_spanned!(
                        arg,
                        "cannot specify `signature_topic` with `anonymous` in ink! event item configuration argument",
                    ));
                }

                if let Some(lit_str) = signature_topic {
                    return Err(duplicate_config_err(
                        lit_str,
                        arg,
                        "signature_topic",
                        "event",
                    ));
                }
                if let Some(lit_str) = arg.value().and_then(ast::MetaValue::as_lit_string)
                {
                    signature_topic = Some(lit_str.clone())
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string literal value for `signature_topic` ink! event item configuration argument",
                    ));
                }
            } else if arg.name().is_ident("name") {
                if let Some(lit_str) = name {
                    return Err(duplicate_config_err(lit_str, arg, "name", "event"));
                }

                if let Some(value) = arg.value() {
                    name = Some(extract_name_override(value, arg.span())?);
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string literal value for `name` attribute argument"
                    ));
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! event item configuration argument",
                ));
            }
        }

        Ok(EventConfig::new(
            anonymous.is_some(),
            signature_topic
                .as_ref()
                .map(SignatureTopic::try_from)
                .transpose()?,
            name.map(|lit_str| lit_str.value()),
        ))
    }
}

impl TryFrom<ast::AttributeArgs> for EventConfig {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        Self::parse(args.into_iter())
    }
}

impl TryFrom<&[syn::Attribute]> for EventConfig {
    type Error = syn::Error;

    fn try_from(attrs: &[syn::Attribute]) -> Result<Self, Self::Error> {
        let mut ink_attrs = Vec::new();
        for attr in attrs {
            if !attr.path().is_ident("ink") {
                continue;
            }
            let nested = attr.parse_args_with(
                syn::punctuated::Punctuated::<ast::Meta, syn::Token![,]>::parse_separated_nonempty,
            )?;
            ink_attrs.extend(nested);
        }
        Self::parse(ink_attrs.into_iter())
    }
}

impl EventConfig {
    /// Construct a new [`EventConfig`].
    pub fn new(
        anonymous: bool,
        signature_topic: Option<SignatureTopic>,
        name: Option<String>,
    ) -> Self {
        Self {
            anonymous,
            signature_topic,
            name,
        }
    }

    /// Returns the anonymous configuration argument.
    pub fn anonymous(&self) -> bool {
        self.anonymous
    }

    /// Returns the manually specified signature topic.
    pub fn signature_topic(&self) -> Option<SignatureTopic> {
        self.signature_topic
    }

    /// Returns the event name override (if any).
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
