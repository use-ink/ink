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

//! A simple mapping to contract storage.
//!
//! # Note
//!
//! This mapping doesn't actually "own" any data.
//! Instead it is just a simple wrapper around the contract storage facilities.

use crate::traits::{
    AutoKey,
    Packed,
    StorableHint,
    StorageKey,
};
use core::marker::PhantomData;
use ink_primitives::Key;
use ink_storage_traits::Storable;
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
/// The mapping requires its own pre-defined storage key where to store values. By
/// default, the key is automatically calculated using [`AutoKey`](crate::traits::AutoKey)
/// during compilation. However, anyone can specify a storage key using
/// [`ManualKey`](crate::traits::ManualKey). Specifying the storage key can be helpful for
/// upgradeable contracts or you want to be resistant to future changes of storage key
/// calculation strategy.
///
/// This is an example of how you can do this:
/// ```rust
/// # #[ink::contract]
/// # mod my_module {
/// use ink::{
///     storage::{
///         traits::ManualKey,
///         Mapping,
///     },
///     U256,
/// };
///
/// #[ink(storage)]
/// #[derive(Default)]
/// pub struct MyContract {
///     balances: Mapping<Address, U256, ManualKey<123>>,
/// }
///
/// impl MyContract {
///     #[ink(constructor)]
///     pub fn new() -> Self {
///         let mut instance = Self::default();
///         let caller = Self::env().caller();
///         let value: U256 = Default::default();
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
/// More usage examples can be found [in the ink! examples](https://github.com/use-ink/ink-examples).
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
        Self::new()
    }
}

impl<K, V, KeyType> Mapping<K, V, KeyType>
where
    V: Packed,
    KeyType: StorageKey,
{
    /// Creates a new empty `Mapping`.
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
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
    ///
    /// Returns the size in bytes of the pre-existing value at the specified key if any.
    ///
    /// # Panics
    ///
    /// Traps if encoding the `key` together with the `value` doesn't fit into the static
    /// buffer.
    #[inline]
    pub fn insert<Q, R>(&mut self, key: Q, value: &R) -> Option<u32>
    where
        Q: scale::EncodeLike<K>,
        R: Storable + scale::EncodeLike<V>,
    {
        ink_env::set_contract_storage(&(&KeyType::KEY, key), value)
    }

    /// Try to insert the given `value` into the mapping under given `key`.
    ///
    /// Fails if `key` or `value` exceeds the static buffer size.
    ///
    /// Returns:
    /// - `Ok(Some(_))` if the value was inserted successfully, containing the size in
    ///   bytes of the pre-existing value at the specified key if any.
    /// - `Ok(None)` if the insert was successful but there was no pre-existing value.
    /// - `Err(_)` if encoding the `key` together with the `value` exceeds the static
    ///   buffer size.
    #[inline]
    pub fn try_insert<Q, R>(&mut self, key: Q, value: &R) -> ink_env::Result<Option<u32>>
    where
        Q: scale::EncodeLike<K>,
        R: Storable + scale::EncodeLike<V>,
    {
        let key_size = <Q as Encode>::encoded_size(&key);

        if key_size > ink_env::BUFFER_SIZE {
            return Err(ink_env::Error::BufferTooSmall)
        }

        let value_size = <R as Storable>::encoded_size(value);

        if key_size.saturating_add(value_size) > ink_env::BUFFER_SIZE {
            return Err(ink_env::Error::BufferTooSmall)
        }

        Ok(self.insert(key, value))
    }

    /// Get the `value` at `key` from the contract storage.
    ///
    /// Returns `None` if no `value` exists at the given `key`.
    ///
    /// # Panics
    ///
    /// Traps if the encoded `key` or `value` doesn't fit into the static buffer.
    #[inline]
    pub fn get<Q>(&self, key: Q) -> Option<V>
    where
        Q: scale::EncodeLike<K>,
    {
        ink_env::get_contract_storage(&(&KeyType::KEY, key))
            .unwrap_or_else(|error| panic!("Failed to get value in Mapping: {error:?}"))
    }

    /// Try to get the `value` at the given `key`.
    ///
    /// Returns:
    /// - `Some(Ok(_))` containing the value if it existed and was decoded successfully.
    /// - `Some(Err(_))` if either (a) the encoded key doesn't fit into the static buffer
    ///   or (b) the value existed but its length exceeds the static buffer size.
    /// - `None` if there was no value under this mapping key.
    #[inline]
    pub fn try_get<Q>(&self, key: Q) -> Option<ink_env::Result<V>>
    where
        Q: scale::EncodeLike<K>,
    {
        let key_size = <Q as Encode>::encoded_size(&key);

        if key_size > ink_env::BUFFER_SIZE {
            return Some(Err(ink_env::Error::BufferTooSmall))
        }

        let value_size: usize =
            ink_env::contains_contract_storage(&(&KeyType::KEY, &key))?
                .try_into()
                .expect("targets of less than 32bit pointer size are not supported; qed");

        if key_size.saturating_add(value_size) > ink_env::BUFFER_SIZE {
            return Some(Err(ink_env::Error::BufferTooSmall))
        }

        self.get(key).map(Ok)
    }

    /// Removes the `value` at `key`, returning the previous `value` at `key` from
    /// storage.
    ///
    /// Returns `None` if no `value` exists at the given `key`.
    ///
    /// # Panics
    ///
    /// Traps if the encoded `key` or `value` doesn't fit into the static buffer.
    #[inline]
    pub fn take<Q>(&self, key: Q) -> Option<V>
    where
        Q: scale::EncodeLike<K>,
    {
        ink_env::take_contract_storage(&(&KeyType::KEY, key))
            .unwrap_or_else(|error| panic!("Failed to take value in Mapping: {error:?}"))
    }

    /// Try to take the `value` at the given `key`.
    /// On success, this operation will remove the value from the mapping
    ///
    /// Returns:
    /// - `Some(Ok(_))` containing the value if it existed and was decoded successfully.
    /// - `Some(Err(_))` if either (a) the encoded key doesn't fit into the static buffer
    ///   or (b) the value existed but its length exceeds the static buffer size.
    /// - `None` if there was no value under this mapping key.
    #[inline]
    pub fn try_take<Q>(&self, key: Q) -> Option<ink_env::Result<V>>
    where
        Q: scale::EncodeLike<K>,
    {
        let key_size = <Q as Encode>::encoded_size(&key);

        if key_size.saturating_add(4 + 32 + 32 + 64 + key_size + 32 + 32)
            > ink_env::remaining_buffer()
        {
            return Some(Err(ink_env::Error::BufferTooSmall))
        }

        let value_size: usize =
            ink_env::contains_contract_storage(&(&KeyType::KEY, &key))?
                .try_into()
                .expect("targets of less than 32bit pointer size are not supported; qed");

        if key_size
            .saturating_add(4 + 32 + 32 + 64 + key_size + 32 + 32)
            .saturating_add(value_size)
            .saturating_add(4 + 32 + 32 + 64 + key_size + 64 + value_size)
            > ink_env::remaining_buffer()
        {
            return Some(Err(ink_env::Error::BufferTooSmall))
        }

        self.take(key).map(Ok)
    }

    /// Get the size in bytes of a value stored at `key` in the contract storage.
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
    /// Returns `false` if no `value` exists at the given `key`.
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

    #[inline]
    fn encoded_size(&self) -> usize {
        0
    }
}

impl<K, V, Key, InnerKey> StorableHint<Key> for Mapping<K, V, InnerKey>
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
                scale_info::meta_type::<Self>(),
            ))
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ManualKey;

    #[test]
    fn insert_and_get_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping: Mapping<u8, _> = Mapping::new();
            mapping.insert(1, &2);
            assert_eq!(mapping.get(1), Some(2));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn insert_and_get_work_for_two_mapping_with_same_manual_key() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping: Mapping<u8, u8, ManualKey<123>> = Mapping::new();
            mapping.insert(1, &2);

            let mapping2: Mapping<u8, u8, ManualKey<123>> = Mapping::new();
            assert_eq!(mapping2.get(1), Some(2));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn gets_default_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mapping: Mapping<u8, u8> = Mapping::new();
            assert_eq!(mapping.get(1), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn insert_and_take_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping: Mapping<u8, _> = Mapping::new();
            mapping.insert(1, &2);
            assert_eq!(mapping.take(1), Some(2));
            assert!(mapping.get(1).is_none());

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn take_empty_value_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mapping: Mapping<u8, u8> = Mapping::new();
            assert_eq!(mapping.take(1), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn can_clear_entries() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // Given
            let mut mapping: Mapping<u8, u8> = Mapping::new();

            mapping.insert(1, &2);
            assert_eq!(mapping.get(1), Some(2));

            // When
            mapping.remove(1);

            // Then
            assert_eq!(mapping.get(1), None);

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
            mapping.remove(1);

            // Then
            assert_eq!(mapping.get(1), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn fallible_storage_works_for_fitting_data() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping: Mapping<u8, [u8; ink_env::BUFFER_SIZE - 1]> = Mapping::new();

            let key = 0;
            let value = [0u8; ink_env::BUFFER_SIZE - 1];

            assert_eq!(mapping.try_insert(key, &value), Ok(None));
            assert_eq!(mapping.try_get(key), Some(Ok(value)));
            assert_eq!(mapping.try_take(key), Some(Ok(value)));
            assert_eq!(mapping.try_get(key), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn fallible_storage_fails_gracefully_for_overgrown_data() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping: Mapping<u8, [u8; ink_env::BUFFER_SIZE]> = Mapping::new();

            let key = 0;
            let value = [0u8; ink_env::BUFFER_SIZE];

            assert_eq!(mapping.try_get(0), None);
            assert_eq!(
                mapping.try_insert(key, &value),
                Err(ink_env::Error::BufferTooSmall)
            );

            // The off-chain impl conveniently uses a Vec for encoding,
            // allowing writing values exceeding the static buffer size.
            ink_env::set_contract_storage(&(&mapping.key(), key), &value);
            assert_eq!(
                mapping.try_get(key),
                Some(Err(ink_env::Error::BufferTooSmall))
            );
            assert_eq!(
                mapping.try_take(key),
                Some(Err(ink_env::Error::BufferTooSmall))
            );

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn fallible_storage_considers_key_size() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping: Mapping<[u8; ink_env::BUFFER_SIZE + 1], u8> = Mapping::new();

            let key = [0u8; ink_env::BUFFER_SIZE + 1];
            let value = 0;

            // Key is already too large, so this should fail anyways.
            assert_eq!(
                mapping.try_insert(key, &value),
                Err(ink_env::Error::BufferTooSmall)
            );

            // The off-chain impl conveniently uses a Vec for encoding,
            // allowing writing values exceeding the static buffer size.
            ink_env::set_contract_storage(&(&mapping.key(), key), &value);
            assert_eq!(
                mapping.try_get(key),
                Some(Err(ink_env::Error::BufferTooSmall))
            );
            assert_eq!(
                mapping.try_take(key),
                Some(Err(ink_env::Error::BufferTooSmall))
            );

            Ok(())
        })
        .unwrap()
    }
}
