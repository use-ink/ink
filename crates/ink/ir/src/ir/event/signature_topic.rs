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

use core::fmt::{
    Display,
    Formatter,
};

use impl_serde::serialize as serde_hex;
use syn::spanned::Spanned;

use crate::ast;

/// The signature topic argument of an event variant.
///
/// Used as part of `ink::event` macro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignatureTopic {
    topic: [u8; 32],
}

impl SignatureTopic {
    /// Returns a 32-byte array representation of the signature topic.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.topic
    }

    /// Returns a 32 byte hex-string representation of the signature topic.
    pub fn to_hex(&self) -> String {
        serde_hex::to_hex(self.topic.as_slice(), false)
    }
}

impl Display for SignatureTopic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for SignatureTopic {
    fn from(value: [u8; 32]) -> Self {
        Self { topic: value }
    }
}

impl TryFrom<&str> for SignatureTopic {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, String> {
        let bytes: [u8; 32] = serde_hex::from_hex(value)
            .map_err(|_| "`signature_topic` has invalid hex string".to_string())?
            .try_into()
            .map_err(|e: Vec<u8>| {
                format!(
                    "`signature_topic` is expected to be 32-byte hex string. \
                    Found {} bytes",
                    e.len()
                )
            })?;

        Ok(Self { topic: bytes })
    }
}

impl TryFrom<&syn::LitStr> for SignatureTopic {
    type Error = syn::Error;

    fn try_from(lit: &syn::LitStr) -> Result<Self, Self::Error> {
        Self::try_from(lit.value().as_str())
            .map_err(|err| syn::Error::new_spanned(lit, err))
    }
}

impl TryFrom<&syn::Lit> for SignatureTopic {
    type Error = syn::Error;

    fn try_from(lit: &syn::Lit) -> Result<Self, Self::Error> {
        if let syn::Lit::Str(s) = lit {
            Self::try_from(s)
        } else {
            Err(format_err_spanned!(
                lit,
                "Expected string literal argument for the `signature_topic`"
            ))
        }
    }
}

impl TryFrom<&ast::MetaValue> for SignatureTopic {
    type Error = syn::Error;

    fn try_from(value: &ast::MetaValue) -> Result<Self, Self::Error> {
        if let ast::MetaValue::Lit(lit) = value {
            Self::try_from(lit)
        } else {
            Err(format_err_spanned!(
                value,
                "Expected string argument for the `signature_topic`"
            ))
        }
    }
}

impl TryFrom<ast::AttributeArgs> for Option<SignatureTopic> {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut signature_topic: Option<SignatureTopic> = None;
        for arg in args.into_iter() {
            if arg.name().is_ident("signature_topic") {
                if signature_topic.is_some() {
                    return Err(format_err!(
                        arg.span(),
                        "encountered duplicate ink! event configuration argument"
                    ));
                }
                signature_topic =
                    arg.value().map(SignatureTopic::try_from).transpose()?;
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! event item configuration argument",
                ));
            }
        }
        Ok(signature_topic)
    }
}

impl TryFrom<&syn::MetaNameValue> for SignatureTopic {
    type Error = syn::Error;

    fn try_from(nv: &syn::MetaNameValue) -> Result<Self, Self::Error> {
        if nv.path.is_ident("signature_topic") {
            if let syn::Expr::Lit(lit_expr) = &nv.value {
                Self::try_from(&lit_expr.lit)
            } else {
                Err(format_err_spanned!(&nv.value, "Expected literal argument"))
            }
        } else {
            Err(format_err_spanned!(nv, "Expected `signature_topic` ident"))
        }
    }
}
