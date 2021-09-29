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

//! A simple mapping to contract storage.

use crate::traits::{
    ExtKeyPtr,
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use ink_env::hash::{
    Blake2x256,
    HashOutput,
};
use ink_primitives::Key;

use core::marker::PhantomData;

/// A mapping of key-value pairs directly into contract storage.
///
/// If a key does not exist the `Default` value for the `value` will be returned.
pub struct Mapping<K, V> {
    key: Key,
    mapping: (PhantomData<K>, PhantomData<V>),
}

impl<K, V> Mapping<K, V>
where
    K: scale::Encode,
    V: scale::Encode + scale::Decode + Default,
{
    /// Insert the given `value` to the contract storage.
    pub fn insert(&mut self, key: K, value: V) {
        ink_env::set_contract_storage(&self.key(key), &value);
    }

    /// Get the `value` at `key` from the contract storage.
    pub fn get(&self, key: K) -> V {
        ink_env::get_contract_storage(&self.key(key))
            .unwrap_or_default()
            .unwrap_or_default()
    }

    fn key(&self, key: K) -> Key {
        let encodedable_key = (self.key, key);
        let mut output = <Blake2x256 as HashOutput>::Type::default();
        ink_env::hash_encoded::<Blake2x256, _>(&encodedable_key, &mut output);
        output.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_to_storage_works() {
        let mut m1 = Mapping::new();
        m1.insert("Hello", "World");
        assert_eq!(m1.get("Hello"), "World");
    }
}
