// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use ink_prelude::vec;
use sha2_const::Sha256;

pub type StorageKey = u32;

/// Contains all rules related to storage key creation.
pub struct StorageKeyComposer;

impl StorageKeyComposer {
    /// Concatenate two `StorageKey` into one. If one of the keys is zero, then return another
    /// without hashing. If both keys are non-zero, return the hash of both keys.
    pub const fn concat(left: StorageKey, right: StorageKey) -> StorageKey {
        match (left, right) {
            (0, 0) => 0,
            (0, _) => right,
            (_, 0) => left,
            (left, right) => {
                let hash = Sha256::new()
                    .update(&left.to_be_bytes())
                    .update(&right.to_be_bytes())
                    .finalize();
                StorageKey::from_be_bytes([hash[0], hash[1], hash[2], hash[3]])
            }
        }
    }

    /// Return storage key from the string
    pub const fn from_str(str: &str) -> StorageKey {
        Self::from_bytes(str.as_bytes())
    }

    /// Return storage key from the bytes
    pub const fn from_bytes(bytes: &[u8]) -> StorageKey {
        let hash = Sha256::new().update(bytes).finalize();
        StorageKey::from_be_bytes([hash[0], hash[1], hash[2], hash[3]])
    }

    /// # Note
    ///
    /// - `variant_name` is `None` for structures and unions.
    /// - if the field is unnamed then `field_name` is `"{}"` where `{}` is a number of the field.
    ///
    /// Evaluates the storage key of the field in the structure, variant or union.
    ///
    /// 1. Compute the ASCII byte representation of `struct_name` and call it `S`.
    /// 1. If `variant_name` is `Some` then computes the ASCII byte representation and call it `V`.
    /// 1. Compute the ASCII byte representation of `field_name` and call it `F`.
    /// 1. Concatenate (`S` and `F`) or (`S`, `V` and `F`) using `::` as separator and call it `C`.
    /// 1. Apply the `SHA2` 256-bit hash `H` of `C`.
    /// 1. The first 4 bytes of `H` make up the storage key.
    pub fn compute_storage_key(
        struct_name: &str,
        variant_name: &str,
        field_name: &str,
    ) -> u32 {
        let separator = &b"::"[..];
        let composed_key = if !variant_name.is_empty() {
            vec![
                struct_name.as_bytes(),
                variant_name.as_bytes(),
                field_name.as_bytes(),
            ]
            .join(separator)
        } else {
            vec![struct_name.as_bytes(), field_name.as_bytes()].join(separator)
        };

        Self::from_bytes(composed_key.as_slice())
    }

    /// Return the 32 bytes representation of the storage key for old version of the storage.
    pub const fn old_key(new_key: &StorageKey) -> [u8; 32] {
        let bytes = new_key.to_le_bytes();
        [
            bytes[0], bytes[1], bytes[2], bytes[3], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]
    }
}
