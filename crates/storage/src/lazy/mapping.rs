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
    AutoKey,
    Item,
    Packed,
    StorageKey,
};
use core::marker::PhantomData;
use ink_primitives::{
    traits::Storable,
    Key,
};
use scale::{
    Encode,
    Error,
    Input,
    Output,
};

/// A mapping of key-value pairs directly into contract storage.
///
/// # Important
///
/// The mapping requires its own pre-defined storage key where to store values. By default,
/// the is automatically calculated using [`AutoKey`](crate::traits::AutoKey) during compilation.
/// However, anyone can specify a storage key using [`ManualKey`](crate::traits::ManualKey).
/// Specifying the storage key can be helpful for upgradeable contracts or you want to be resistant
/// to future changes of storage key calculation strategy.
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
/// use ink_storage::{traits::ManualKey, Mapping};
///
/// #[ink(storage)]
/// #[derive(Default)]
/// pub struct MyContract {
///     balances: Mapping<AccountId, Balance, ManualKey<123>>,
/// }
///
/// impl MyContract {
///     #[ink(constructor)]
///     pub fn new() -> Self {
///         let mut instance = Self::default();
///         let caller = Self::env().caller();
///         let value: Balance = Default::default();
///         instance.balances.insert(&caller, &value);
///         instance
///     }
///
/// #   #[ink(message)]
/// #   pub fn my_message(&self) { }
/// }
/// # }
/// ```
///
/// More usage examples can be found [in the ink! examples](https://github.com/paritytech/ink/tree/master/examples).
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Mapping<K, V: Packed, KeyType: StorageKey = AutoKey> {
    #[allow(clippy::type_complexity)]
    _marker: PhantomData<fn() -> (K, V, KeyType)>,
}

/// We implement this manually because the derived implementation adds trait bounds.
impl<K, V, KeyType> Default for Mapping<K, V, KeyType>
where
    V: Packed,
    KeyType: StorageKey,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<K, V, KeyType> Mapping<K, V, KeyType>
where
    V: Packed,
    KeyType: StorageKey,
{
    /// Creates a new empty `Mapping`.
    pub fn new() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<K, V, KeyType> ::core::fmt::Debug for Mapping<K, V, KeyType>
where
    V: Packed,
    KeyType: StorageKey,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Mapping")
            .field("key", &KeyType::KEY)
            .finish()
    }
}

impl<K, V, KeyType> Mapping<K, V, KeyType>
where
    K: Encode,
    V: Packed,
    KeyType: StorageKey,
{
    /// Insert the given `value` to the contract storage.
    #[inline]
    pub fn insert<Q, R>(&mut self, key: Q, value: &R)
    where
        Q: scale::EncodeLike<K>,
        R: Storable + scale::EncodeLike<V>,
    {
        ink_env::set_contract_storage(&(&KeyType::KEY, key), value);
    }

    /// Insert the given `value` to the contract storage.
    ///
    /// Returns the size of the pre-existing value at the specified key if any.
    #[inline]
    pub fn insert_return_size<Q, R>(&mut self, key: Q, value: &R) -> Option<u32>
    where
        Q: scale::EncodeLike<K>,
        R: Storable + scale::EncodeLike<V>,
    {
        ink_env::set_contract_storage(&(&KeyType::KEY, key), value)
    }

    /// Get the `value` at `key` from the contract storage.
    ///
    /// Returns `None` if no `value` exists at the given `key`.
    #[inline]
    pub fn get<Q>(&self, key: Q) -> Option<V>
    where
        Q: scale::EncodeLike<K>,
    {
        ink_env::get_contract_storage(&(&KeyType::KEY, key))
            .unwrap_or_else(|error| panic!("Failed to get value in Mapping: {:?}", error))
    }

    /// Get the size of a value stored at `key` in the contract storage.
    ///
    /// Returns `None` if no `value` exists at the given `key`.
    #[inline]
    pub fn size<Q>(&self, key: Q) -> Option<u32>
    where
        Q: scale::EncodeLike<K>,
    {
        ink_env::contains_contract_storage(&(&KeyType::KEY, key))
    }

    /// Checks if a value is stored at the given `key` in the contract storage.
    ///
    /// Returns `None` if no `value` exists at the given `key`.
    #[inline]
    pub fn contains<Q>(&self, key: Q) -> bool
    where
        Q: scale::EncodeLike<K>,
    {
        ink_env::contains_contract_storage(&(&KeyType::KEY, key)).is_some()
    }

    /// Clears the value at `key` from storage.
    #[inline]
    pub fn remove<Q>(&self, key: Q)
    where
        Q: scale::EncodeLike<K>,
    {
        ink_env::clear_contract_storage(&(&KeyType::KEY, key));
    }
}

impl<K, V, KeyType> Storable for Mapping<K, V, KeyType>
where
    V: Packed,
    KeyType: StorageKey,
{
    #[inline]
    fn encode<T: Output + ?Sized>(&self, _dest: &mut T) {}

    #[inline]
    fn decode<I: Input>(_input: &mut I) -> Result<Self, Error> {
        Ok(Default::default())
    }
}

impl<K, V, Key, InnerKey> Item<Key> for Mapping<K, V, InnerKey>
where
    V: Packed,
    Key: StorageKey,
    InnerKey: StorageKey,
{
    type Type = Mapping<K, V, Key>;
    type PreferredKey = InnerKey;
}

impl<K, V, KeyType> StorageKey for Mapping<K, V, KeyType>
where
    V: Packed,
    KeyType: StorageKey,
{
    const KEY: Key = KeyType::KEY;
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::{
        Layout,
        LayoutKey,
        RootLayout,
    };

    impl<K, V, KeyType> StorageLayout for Mapping<K, V, KeyType>
    where
        K: scale_info::TypeInfo + 'static,
        V: Packed + StorageLayout + scale_info::TypeInfo + 'static,
        KeyType: StorageKey + scale_info::TypeInfo + 'static,
    {
        fn layout(_: &Key) -> Layout {
            Layout::Root(RootLayout::new(
                LayoutKey::from(&KeyType::KEY),
                <V as StorageLayout>::layout(&KeyType::KEY),
            ))
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping: Mapping<u8, _> = Mapping::new();
            mapping.insert(&1, &2);
            assert_eq!(mapping.get(&1), Some(2));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn gets_default_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mapping: Mapping<u8, u8> = Mapping::new();
            assert_eq!(mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn can_clear_entries() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // Given
            let mut mapping: Mapping<u8, u8> = Mapping::new();

            mapping.insert(&1, &2);
            assert_eq!(mapping.get(&1), Some(2));

            // When
            mapping.remove(&1);

            // Then
            assert_eq!(mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn can_clear_unexistent_entries() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // Given
            let mapping: Mapping<u8, u8> = Mapping::new();

            // When
            mapping.remove(&1);

            // Then
            assert_eq!(mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }
}
