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
//!
//! # Note
//!
//! This mapping doesn't actually "own" any data.
//! Instead it is just a simple wrapper around the contract storage facilities.

use crate::traits::{
    pull_packed_root_opt,
    push_packed_root,
    ExtKeyPtr,
    KeyPtr,
    PackedLayout,
    SpreadAllocate,
    SpreadLayout,
};
use core::marker::PhantomData;

use ink_env::hash::{
    Blake2x256,
    HashOutput,
};
use ink_primitives::Key;

/// A mapping of key-value pairs directly into contract storage.
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[derive(Default)]
pub struct Mapping<K, V> {
    offset_key: Key,
    _marker: PhantomData<fn() -> (K, V)>,
}

impl<K, V> core::fmt::Debug for Mapping<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Mapping")
            .field("offset_key", &self.offset_key)
            .finish()
    }
}

impl<K, V> Mapping<K, V> {
    /// Creates a new empty `Mapping`.
    fn new(offset_key: Key) -> Self {
        Self {
            offset_key,
            _marker: Default::default(),
        }
    }
}

impl<K, V> Mapping<K, V>
where
    K: PackedLayout,
    V: PackedLayout,
{
    /// Insert the given `value` to the contract storage.
    #[inline]
    pub fn insert<Q, R>(&mut self, key: Q, value: &R)
    where
        Q: scale::EncodeLike<K>,
        R: scale::EncodeLike<V> + PackedLayout,
    {
        push_packed_root(value, &self.storage_key(&key));
    }

    /// Get the `value` at `key` from the contract storage.
    ///
    /// Returns `None` if no `value` exists at the given `key`.
    #[inline]
    pub fn get<Q>(&self, key: Q) -> Option<V>
    where
        Q: scale::EncodeLike<K>,
    {
        pull_packed_root_opt(&self.storage_key(&key))
    }

    /// Clears the value at `key` from storage.
    pub fn clear_entry<Q>(&self, key: Q)
    where
        Q: scale::EncodeLike<K>,
    {
        let storage_key = self.storage_key(&key);
        if <V as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP {
            // There are types which need to perform some action before being cleared. Here we
            // indicate to those types that they should start tidying up.
            if let Some(value) = self.get(key) {
                <V as PackedLayout>::clear_packed(&value, &storage_key);
            }
        }
        ink_env::clear_contract_storage(&storage_key);
    }

    /// Returns a `Key` pointer used internally by the storage API.
    ///
    /// This key is a combination of the `Mapping`'s internal `offset_key`
    /// and the user provided `key`.
    fn storage_key<Q>(&self, key: &Q) -> Key
    where
        Q: scale::EncodeLike<K>,
    {
        let encodedable_key = (&self.offset_key, key);
        let mut output = <Blake2x256 as HashOutput>::Type::default();
        ink_env::hash_encoded::<Blake2x256, _>(&encodedable_key, &mut output);
        output.into()
    }
}

impl<K, V> SpreadLayout for Mapping<K, V> {
    const FOOTPRINT: u64 = 1;
    const REQUIRES_DEEP_CLEAN_UP: bool = false;

    #[inline]
    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        // Note: There is no need to pull anything from the storage for the
        //       mapping type since it initializes itself entirely by the
        //       given key pointer.
        Self::new(*ExtKeyPtr::next_for::<Self>(ptr))
    }

    #[inline]
    fn push_spread(&self, ptr: &mut KeyPtr) {
        // Note: The mapping type does not store any state in its associated
        //       storage region, therefore only the pointer has to be incremented.
        ptr.advance_by(Self::FOOTPRINT);
    }

    #[inline]
    fn clear_spread(&self, ptr: &mut KeyPtr) {
        // Note: The mapping type is not aware of its elements, therefore
        //       it is not possible to clean up after itself.
        ptr.advance_by(Self::FOOTPRINT);
    }
}

impl<K, V> SpreadAllocate for Mapping<K, V> {
    #[inline]
    fn allocate_spread(ptr: &mut KeyPtr) -> Self {
        // Note: The mapping type initializes itself entirely by the key pointer.
        Self::new(*ExtKeyPtr::next_for::<Self>(ptr))
    }
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::{
        CellLayout,
        Layout,
        LayoutKey,
    };

    impl<K, V> StorageLayout for Mapping<K, V>
    where
        K: scale_info::TypeInfo + 'static,
        V: scale_info::TypeInfo + 'static,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Cell(CellLayout::new::<Self>(LayoutKey::from(
                key_ptr.advance_by(1),
            )))
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    /// A dummy type which `REQUIRES_DEEP_CLEAN_UP`
    #[derive(PartialEq, Debug, scale::Encode, scale::Decode)]
    struct DeepClean<T>(T);

    impl<T> SpreadLayout for DeepClean<T>
    where
        T: PackedLayout,
    {
        const FOOTPRINT: u64 = 1;
        const REQUIRES_DEEP_CLEAN_UP: bool = true;

        fn pull_spread(ptr: &mut KeyPtr) -> Self {
            DeepClean(<T as SpreadLayout>::pull_spread(ptr))
        }

        fn push_spread(&self, ptr: &mut KeyPtr) {
            <T as SpreadLayout>::push_spread(&self.0, ptr)
        }

        fn clear_spread(&self, ptr: &mut KeyPtr) {
            <T as SpreadLayout>::clear_spread(&self.0, ptr)
        }
    }

    impl<T> PackedLayout for DeepClean<T>
    where
        T: PackedLayout + scale::EncodeLike<T>,
    {
        fn pull_packed(&mut self, at: &Key) {
            <T as PackedLayout>::pull_packed(&mut self.0, at);
        }
        fn push_packed(&self, at: &Key) {
            <T as PackedLayout>::push_packed(&self.0, at);
        }
        fn clear_packed(&self, at: &Key) {
            <T as PackedLayout>::clear_packed(&self.0, at);
        }
    }

    #[test]
    fn insert_and_get_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping: Mapping<u8, _> = Mapping::new([0u8; 32].into());
            mapping.insert(&1, &2);
            assert_eq!(mapping.get(&1), Some(2));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn gets_default_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mapping: Mapping<u8, u8> = Mapping::new([0u8; 32].into());
            assert_eq!(mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn can_clear_entries() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // Given
            let mut mapping: Mapping<u8, u8> = Mapping::new([0u8; 32].into());
            let mut deep_mapping: Mapping<u8, DeepClean<u8>> =
                Mapping::new([1u8; 32].into());

            mapping.insert(&1, &2);
            assert_eq!(mapping.get(&1), Some(2));

            deep_mapping.insert(&1, &DeepClean(2));
            assert_eq!(deep_mapping.get(&1), Some(DeepClean(2)));

            // When
            mapping.clear_entry(&1);
            deep_mapping.clear_entry(&1);

            // Then
            assert_eq!(mapping.get(&1), None);
            assert_eq!(deep_mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn can_clear_unexistent_entries() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // Given
            let mapping: Mapping<u8, u8> = Mapping::new([0u8; 32].into());
            let deep_mapping: Mapping<u8, DeepClean<u8>> = Mapping::new([1u8; 32].into());

            // When
            mapping.clear_entry(&1);
            deep_mapping.clear_entry(&1);

            // Then
            assert_eq!(mapping.get(&1), None);
            assert_eq!(deep_mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }
}
