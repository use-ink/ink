// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use core::fmt::Write;

/// Serializes the given bytes as byte string.
pub fn serialize_as_byte_str<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if bytes.is_empty() {
        // Return empty string without prepended `0x`.
        return serializer.serialize_str("")
    }
    let mut hex = String::with_capacity(bytes.len() * 2 + 2);
    write!(hex, "0x").expect("failed writing to string");
    for byte in bytes {
        write!(hex, "{:02x}", byte).expect("failed writing to string");
    }
    serializer.serialize_str(&hex)
}
