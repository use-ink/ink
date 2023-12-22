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

use hex::decode_to_slice;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::ToTokens;
use syn::spanned::Spanned;

use crate::{
    ast,
    error::ExtError,
    utils::extract_cfg_attributes,
};

/// The signature topic argument of an event variant.
///
/// Used as part of `ink::event` macro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignatureTopicArg {
    topic: [u8; 32],
}

impl SignatureTopicArg {
    pub fn signature_topic(&self) -> [u8; 32] {
        self.topic
    }
}

impl From<&[u8; 32]> for SignatureTopicArg {
    fn from(value: &[u8; 32]) -> Self {
        Self { topic: *value }
    }
}

impl TryFrom<&ast::MetaValue> for SignatureTopicArg {
    type Error = syn::Error;

    fn try_from(value: &ast::MetaValue) -> Result<Self, Self::Error> {
        if let ast::MetaValue::Lit(lit) = value {
            if let syn::Lit::Str(s) = lit {
                let mut bytes = [0u8; 32];

                if decode_to_slice(s.value(), &mut bytes).is_ok() {
                    Ok(Self { topic: bytes })
                } else {
                    Err(format_err_spanned!(
                        value,
                        "`signature_topic` has invalid hex string",
                    ))
                }
            } else {
                Err(format_err_spanned!(
                    value,
                    "Expected string literal argument for the `signature_topic`"
                ))
            }
        } else {
            Err(format_err_spanned!(
                value,
                "Expected string argument for the `signature_topic`"
            ))
        }
    }
}

impl TryFrom<ast::AttributeArgs> for Option<SignatureTopicArg> {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut signature_topic: Option<SignatureTopicArg> = None;
        for arg in args.into_iter() {
            if arg.name.is_ident("hash") {
                if signature_topic.is_some() {
                    return Err(format_err!(
                        arg.span(),
                        "encountered duplicate ink! event configuration argument"
                    ));
                }
                signature_topic = Some(SignatureTopicArg::try_from(&arg.value)?);
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! storage item configuration argument",
                ));
            }
        }
        Ok(signature_topic)
    }
}

/// The signature topic argument of an event variant.
///
/// Used as part of `ink::signature_topic` macro.
///
/// Calculated with `blake2b("Event(field1_type,field2_type)")`.
#[derive(Debug, PartialEq, Eq)]
pub struct SignatureTopic {
    item: syn::ItemStruct,
    arg: Option<SignatureTopicArg>,
}

impl SignatureTopic {
    pub fn new(config: TokenStream2, item: TokenStream2) -> Result<Self, syn::Error> {
        let item = syn::parse2::<syn::ItemStruct>(item.clone()).map_err(|err| {
            err.into_combine(format_err_spanned!(
                item,
                "event definition must be a `struct`",
            ))
        })?;
        let parsed_config = syn::parse2::<crate::ast::AttributeArgs>(config)?;
        let arg = Option::<SignatureTopicArg>::try_from(parsed_config)?;

        for attr in &item.attrs {
            if attr
                .path()
                .to_token_stream()
                .to_string()
                .contains("signature_topic")
            {
                return Err(format_err_spanned!(
                    attr,
                    "only one `ink::signature_topic` is allowed",
                ));
            }
        }
        Ok(Self { item, arg })
    }

    /// Returns the event definition .
    pub fn item(&self) -> &syn::ItemStruct {
        &self.item
    }

    /// Return a signature topic, if required.
    pub fn signature_topic(&self) -> Option<[u8; 32]> {
        self.arg.map(|a| a.signature_topic())
    }

    /// Returns a list of `cfg` attributes if any.
    pub fn get_cfg_attrs(&self, span: Span) -> Vec<TokenStream2> {
        extract_cfg_attributes(&self.item.attrs, span)
    }
}
