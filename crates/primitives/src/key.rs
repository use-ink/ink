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

use xxhash_rust::const_xxh32::xxh32;

/// The value 0 is a valid seed.
const XXH32_SEED: u32 = 0;

/// A key into the smart contract storage.
///
/// # Note
///
/// - The storage of an ink! smart contract can be viewed as a key-value store.
/// - In order to manipulate its storage an ink! smart contract is required to indicate
///   the respective cells using this primitive type.
/// - The `Key` type can be compared to a raw pointer and also allows operations similar
///   to pointer arithmetic.
pub type Key = u32;

/// Contains all rules related to storage key creation.
pub struct KeyComposer;

impl KeyComposer {
    /// Concatenate two `Key` into one during compilation.
    pub const fn concat(left: Key, right: Key) -> Key {
        // If one of the keys is zero, then return another without hashing.
        // If both keys are non-zero, return the hash of the XOR difference of both keys.
        match (left, right) {
            (0, 0) => 0,
            (0, _) => right,
            (_, 0) => left,
            (left, right) => xxh32(&(left ^ right).to_be_bytes(), XXH32_SEED),
        }
    }

    /// Return the storage key from the supplied `str`.
    pub const fn from_str(str: &str) -> Key {
        Self::from_bytes(str.as_bytes())
    }

    /// Returns the storage key from the supplied `bytes`.
    pub const fn from_bytes(bytes: &[u8]) -> Key {
        if bytes.is_empty() {
            return 0
        }

        xxh32(bytes, XXH32_SEED)
    }

    /// Evaluates the storage key of the field in the structure, variant or union.
    ///
    /// 1. Compute the ASCII byte representation of `struct_name` and call it `S`.
    /// 1. If `variant_name` is not empty then computes the ASCII byte representation and
    ///    call it `V`. 1. Compute the ASCII byte representation of `field_name` and call
    ///    it `F`. 1. Concatenate (`S` and `F`) or (`S`, `V` and `F`) using `::` as
    ///    separator and call it `C`. 1. The `XXH32` hash of `C` is the storage key.
    ///
    /// # Note
    ///
    /// - `variant_name` is empty for structures and unions.
    /// - if the field is unnamed then `field_name` is `"{}"` where `{}` is a number of
    ///   the field.
    pub fn compute_key(
        struct_name: &str,
        variant_name: &str,
        field_name: &str,
    ) -> Result<Key, Error> {
        if struct_name.is_empty() {
            return Err(Error::StructNameIsEmpty)
        }
        if field_name.is_empty() {
            return Err(Error::FieldNameIsEmpty)
        }

        let separator = &b"::"[..];
        let composed_key = if !variant_name.is_empty() {
            [
                struct_name.as_bytes(),
                variant_name.as_bytes(),
                field_name.as_bytes(),
            ]
            .join(separator)
        } else {
            [struct_name.as_bytes(), field_name.as_bytes()].join(separator)
        };

        Ok(Self::from_bytes(composed_key.as_slice()))
    }
}

/// Possible errors during the computation of the storage key.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    StructNameIsEmpty,
    FieldNameIsEmpty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concat_works_correct() {
        assert_eq!(KeyComposer::concat(0, 13), 13);
        assert_eq!(KeyComposer::concat(31, 0), 31);
        assert_eq!(KeyComposer::concat(31, 13), 0x9ab19a67);
        assert_eq!(KeyComposer::concat(0, 0), 0);
    }

    #[test]
    fn from_str_works_correct() {
        assert_eq!(KeyComposer::from_str(""), 0);
        assert_eq!(KeyComposer::from_str("123"), 0xb6855437);
        assert_eq!(KeyComposer::from_str("Hello world"), 0x9705d437);
    }

    #[test]
    fn from_bytes_works_correct() {
        assert_eq!(KeyComposer::from_bytes(b""), 0);
        assert_eq!(KeyComposer::from_bytes(b"123"), 0xb6855437);
        assert_eq!(KeyComposer::from_bytes(b"Hello world"), 0x9705d437);
    }

    #[test]
    fn compute_key_works_correct() {
        assert_eq!(
            KeyComposer::compute_key("Contract", "", "balances"),
            Ok(0xf820ff02)
        );
        assert_eq!(
            KeyComposer::compute_key("Enum", "Variant", "0"),
            Ok(0x14786b51)
        );
        assert_eq!(
            KeyComposer::compute_key("", "Variant", "0"),
            Err(Error::StructNameIsEmpty)
        );
        assert_eq!(
            KeyComposer::compute_key("Enum", "Variant", ""),
            Err(Error::FieldNameIsEmpty)
        );
    }
}
