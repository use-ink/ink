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

use crate::{
    ast,
    utils::{
        duplicate_config_err,
        extract_sol_name,
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
    signature_topic_hex: Option<String>,
    /// An optional event name override to use for computing the Solidity ABI selector
    /// of the event.
    ///
    /// # Note
    ///
    /// - This configuration argument is only allowed in a Solidity compatible ABI mode
    ///   (i.e. when the ABI mode is "sol" or "all").
    /// - If provided, the name must be a valid Solidity identifier.
    ///
    /// Ref: <https://docs.soliditylang.org/en/latest/grammar.html#a4.SolidityLexer.Identifier>
    sol_name: Option<String>,
}

impl TryFrom<ast::AttributeArgs> for EventConfig {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut anonymous: Option<syn::Path> = None;
        let mut signature_topic: Option<syn::LitStr> = None;
        #[cfg_attr(not(any(ink_abi = "sol", ink_abi = "all")), allow(unused_mut))]
        let mut sol_name: Option<syn::LitStr> = None;
        for arg in args.into_iter() {
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
            } else if arg.name().is_ident("sol_name") {
                #[cfg(any(ink_abi = "sol", ink_abi = "all"))]
                if let Some(lit_str) = sol_name {
                    return Err(duplicate_config_err(lit_str, arg, "sol_name", "event"));
                }

                sol_name = Some(extract_sol_name(arg.value(), arg.span())?);
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! event item configuration argument",
                ));
            }
        }

        Ok(EventConfig::new(
            anonymous.is_some(),
            signature_topic.map(|lit_str| lit_str.value()),
            sol_name.map(|lit_str| lit_str.value()),
        ))
    }
}

impl EventConfig {
    /// Construct a new [`EventConfig`].
    pub fn new(
        anonymous: bool,
        signature_topic_hex: Option<String>,
        sol_name: Option<String>,
    ) -> Self {
        Self {
            anonymous,
            signature_topic_hex,
            sol_name,
        }
    }

    /// Returns the anonymous configuration argument.
    pub fn anonymous(&self) -> bool {
        self.anonymous
    }

    /// Returns the manually specified signature topic.
    pub fn signature_topic_hex(&self) -> Option<&str> {
        self.signature_topic_hex.as_deref()
    }

    /// Returns the event name override (if any) for computing the Solidity ABI selector
    /// of the event.
    pub fn sol_name(&self) -> Option<&str> {
        self.sol_name.as_deref()
    }
}
