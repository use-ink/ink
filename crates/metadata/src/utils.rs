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

use crate::serde_hex;

/// Serializes the given bytes as byte string.
pub fn serialize_as_byte_str<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if bytes.is_empty() {
        // Return empty string without prepended `0x`.
        return serializer.serialize_str("")
    }
    serde_hex::serialize(bytes, serializer)
}

/// Deserializes the given hex string with optional `0x` prefix.
pub fn deserialize_from_byte_str<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;

    impl<'b> serde::de::Visitor<'b> for Visitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "hex string with optional 0x prefix")
        }

        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            let result = if v.starts_with("0x") {
                serde_hex::from_hex(v)
            } else {
                serde_hex::from_hex(&format!("0x{}", v))
            };
            result.map_err(E::custom)
        }

        fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
            self.visit_str(&v)
        }
    }

    deserializer.deserialize_str(Visitor)
}
