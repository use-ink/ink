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
///
/// # Important
///
/// If you use this data structure you must use the function
/// [`ink_lang::utils::initialize_contract`](https://paritytech.github.io/ink/ink_lang/utils/fn.initialize_contract.html)
/// in your contract's constructors!
///
/// Note that in order to use this function your contract's storage struct must implement the
/// [`SpreadAllocate`](crate::traits::SpreadAllocate) trait.
///
/// This is an example of how you can do this:
/// ```rust
/// # use ink_lang as ink;
/// # use ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// # };
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
///
/// # #[ink::contract]
/// # mod my_module {
/// use ink_storage::{traits::SpreadAllocate, Mapping};
///
/// #[ink(storage)]
/// #[derive(SpreadAllocate)]
/// pub struct MyContract {
///     balances: Mapping<AccountId, Balance>,
/// }
///
/// impl MyContract {
///     #[ink(constructor)]
///     pub fn new() -> Self {
///         ink_lang::utils::initialize_contract(Self::new_init)
///     }
///
///     /// Default initializes the contract.
///     fn new_init(&mut self) {
///         let caller = Self::env().caller();
///         let value: Balance = Default::default();
///         self.balances.insert(&caller, &value);
///     }
/// #   #[ink(message)]
/// #   pub fn my_message(&self) { }
/// }
/// # }
/// ```
///
/// More usage examples can be found [in the ink! examples](https://github.com/paritytech/ink/tree/master/examples).
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Mapping<K, V> {
    offset_key: Key,
    _marker: PhantomData<fn() -> (K, V)>,
}

/// We implement this manually because the derived implementation adds trait bounds.
impl<K, V> Default for Mapping<K, V> {
    fn default() -> Self {
        Self {
            offset_key: Default::default(),
            _marker: Default::default(),
        }
    }
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

    /// Insert the given `value` to the contract storage.
    ///
    /// Returns the size of the pre-existing value at the specified key if any.
    #[inline]
    pub fn insert_return_size<Q, R>(&mut self, key: Q, value: &R) -> Option<u32>
    where
        Q: scale::EncodeLike<K>,
        R: scale::EncodeLike<V> + PackedLayout,
    {
        push_packed_root(value, &self.storage_key(&key))
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

    /// Get the size of a value stored at `key` in the contract storage.
    ///
    /// Returns `None` if no `value` exists at the given `key`.
    #[inline]
    pub fn size<Q>(&self, key: Q) -> Option<u32>
    where
        Q: scale::EncodeLike<K>,
    {
        ink_env::contract_storage_contains(&self.storage_key(&key))
    }

    /// Checks if a value is stored at the given `key` in the contract storage.
    ///
    /// Returns `None` if no `value` exists at the given `key`.
    #[inline]
    pub fn contains<Q>(&self, key: Q) -> bool
    where
        Q: scale::EncodeLike<K>,
    {
        ink_env::contract_storage_contains(&self.storage_key(&key)).is_some()
    }

    /// Clears the value at `key` from storage.
    pub fn remove<Q>(&self, key: Q)
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
            // We use `Pack` here since it `REQUIRES_DEEP_CLEAN_UP`
            use crate::Pack;

            // Given
            let mut mapping: Mapping<u8, u8> = Mapping::new([0u8; 32].into());
            let mut deep_mapping: Mapping<u8, Pack<u8>> = Mapping::new([1u8; 32].into());

            mapping.insert(&1, &2);
            assert_eq!(mapping.get(&1), Some(2));

            deep_mapping.insert(&1u8, &Pack::new(Pack::new(2u8)));
            assert_eq!(deep_mapping.get(&1), Some(Pack::new(2u8)));

            // When
            mapping.remove(&1);
            deep_mapping.remove(&1);

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
            // We use `Pack` here since it `REQUIRES_DEEP_CLEAN_UP`
            use crate::Pack;

            // Given
            let mapping: Mapping<u8, u8> = Mapping::new([0u8; 32].into());
            let deep_mapping: Mapping<u8, Pack<u8>> = Mapping::new([1u8; 32].into());

            // When
            mapping.remove(&1);
            deep_mapping.remove(&1);

            // Then
            assert_eq!(mapping.get(&1), None);
            assert_eq!(deep_mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }
}
