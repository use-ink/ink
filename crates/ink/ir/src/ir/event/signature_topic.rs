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

use crate::ast;

/// The signature topic of an event variant.
///
/// Calculated with `blake2b("Event(field1_type,field2_type)")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignatureTopic {
    topic: [u8; 32],
}

impl SignatureTopic {
    pub fn signature_topic(&self) -> [u8; 32] {
        self.topic
    }
}

impl From<&[u8; 32]> for SignatureTopic {
    fn from(value: &[u8; 32]) -> Self {
        Self { topic: *value }
    }
}

impl TryFrom<&ast::MetaValue> for SignatureTopic {
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
