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

//! A simple storage vector implementation built on top of [Mapping].
//!
//! # Note
//!
//! This vector doesn't actually "own" any data.
//! Instead it is just a simple wrapper around the contract storage facilities.

use core::cell::Cell;
use ink_primitives::Key;
use ink_storage_traits::{
    AutoKey,
    Packed,
    Storable,
    StorableHint,
    StorageKey,
};
use scale::EncodeLike;

use crate::{
    Lazy,
    Mapping,
};

/// A vector of values (elements) directly on contract storage.
///
/// # Important
///
/// [StorageVec] requires its own pre-defined storage key where to store values. By
/// default, the is automatically calculated using [`AutoKey`](crate::traits::AutoKey)
/// during compilation. However, anyone can specify a storage key using
/// [`ManualKey`](crate::traits::ManualKey). Specifying the storage key can be helpful for
/// upgradeable contracts or you want to be resistant to future changes of storage key
/// calculation strategy.
///
/// # Differences between `ink::prelude::vec::Vec` and [StorageVec]
///
/// Any `Vec<T>` will exhibit [Packed] storage layout; where
/// [StorageVec] stores each value under it's own storage key.
///
/// Hence, any read or write from or to a `Vec` on storage will load
/// or store _all_ of its elements.
///
/// This can be undesirable:
/// The cost of reading or writing a _single_ element grows linearly
/// corresponding to the number of elements in the vector (its length).
/// Additionally, the maximum capacity of the _whole_ vector is limited by
/// the size of the static buffer used during ABI encoding and decoding
/// (default 16 KiB).
///
/// [StorageVec] on the other hand allows to access each element individually.
/// Thus, it can theoretically grow to infinite size.
/// However, we currently limit the length at 2 ^ 32 elements. In practice,
/// even if the vector elements are single bytes, it'll allow to store
/// more than 4 GB data in blockchain storage.
///
/// # Caveats
///
/// Iterators are not provided. [StorageVec] is expected to be used to
/// store a lot elements, where iterating through the elements would be
/// rather inefficient (naturally, it is still possible to manually
/// iterate over the elements using a loop).
///
/// For the same reason, operations which would require re-ordering
/// stored elements are not supported. Examples include inserting and
/// deleting elements at arbitrary positions or sorting elements.
///
/// The decision whether to use `Vec<T>` or [StorageVec] can be seen as an
/// optimization problem with several factors:
/// * How large you expect the vector to grow
/// * The size of individual elements being stored
/// * How frequently reads, writes and iterations happen
///
/// For example, if a vector is expected to stay small but is frequently
/// iterated over. Choosing a `Vec<T>` instead of [StorageVec] will be
/// preferred as individual storage reads are much more expensive as
/// opposed to retrieving and decoding the whole collection with a single
/// storage read.
///
/// # Storage Layout
///
/// At given [StorageKey] `K`, the length of the [StorageVec] is hold.
/// Each element `E` is then stored under a combination of the [StorageVec]
/// key `K` and the elements index.
///
/// Given [StorageVec] under key `K`, the storage key `E` of the `N`th
/// element is calculated as follows:
///
/// `E = scale::Encode((K, N))`
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct StorageVec<V: Packed, KeyType: StorageKey = AutoKey> {
    /// The number of elements stored on-chain.
    ///
    /// # Note
    ///
    /// Because of caching, never operate on this field directly!
    /// Always use `fn get_len()` an `fn set_len()` instead.
    len: Lazy<u32, KeyType>,
    /// The length only changes upon pushing to or popping from the vec.
    /// Hence we can cache it to prevent unnecessary reads from storage.
    ///
    /// # Note
    ///
    /// Because of caching, never operate on this field directly!
    /// Always use `fn get_len()` an `fn set_len()` instead.
    #[cfg_attr(feature = "std", codec(skip))]
    len_cached: CachedLen,
    /// We use a [Mapping] to store all elements of the vector.
    /// Each element is living in storage under `&(KeyType::KEY, index)`.
    /// Because [StorageVec] has a [StorageKey] parameter under which the
    /// length and element are stored, it won't collide with the other
    /// storage fields (unless contract authors purposefully craft such a
    /// storage layout).
    elements: Mapping<u32, V, KeyType>,
}

#[derive(Debug)]
struct CachedLen(Cell<Option<u32>>);

impl<V, KeyType> Default for StorageVec<V, KeyType>
where
    V: Packed,
    KeyType: StorageKey,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V, KeyType> Storable for StorageVec<V, KeyType>
where
    V: Packed,
    KeyType: StorageKey,
{
    #[inline]
    fn encode<T: scale::Output + ?Sized>(&self, _dest: &mut T) {}

    #[inline]
    fn decode<I: scale::Input>(_input: &mut I) -> Result<Self, scale::Error> {
        Ok(Default::default())
    }

    #[inline]
    fn encoded_size(&self) -> usize {
        0
    }
}

impl<V, Key, InnerKey> StorableHint<Key> for StorageVec<V, InnerKey>
where
    V: Packed,
    Key: StorageKey,
    InnerKey: StorageKey,
{
    type Type = StorageVec<V, Key>;
    type PreferredKey = InnerKey;
}

impl<V, KeyType> StorageKey for StorageVec<V, KeyType>
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

    impl<V, KeyType> StorageLayout for StorageVec<V, KeyType>
    where
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

impl<V, KeyType> StorageVec<V, KeyType>
where
    V: Packed,
    KeyType: StorageKey,
{
    /// Creates a new empty `StorageVec`.
    pub const fn new() -> Self {
        Self {
            len: Lazy::new(),
            len_cached: CachedLen(Cell::new(None)),
            elements: Mapping::new(),
        }
    }

    /// Returns the number of elements in the vector, also referred to as its length.
    ///
    /// The length is cached; subsequent calls (without writing to the vector) won't
    /// trigger additional storage reads.
    #[inline]
    pub fn len(&self) -> u32 {
        let cached_len = self.len_cached.0.get();

        debug_assert!(cached_len.is_none() || self.len.get() == cached_len);

        cached_len.unwrap_or_else(|| {
            let value = self.len.get();
            self.len_cached.0.set(value);
            value.unwrap_or(u32::MIN)
        })
    }

    /// Overwrite the length. Writes directly to contract storage.
    fn set_len(&mut self, new_len: u32) {
        self.len.set(&new_len);
        self.len_cached.0.set(Some(new_len));
    }

    /// Returns `true` if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Appends an element to the back of the vector.
    ///
    /// # Panics
    ///
    /// * If the vector is at capacity (max. of 2 ^ 32 elements).
    /// * If the value overgrows the static buffer size.
    /// * If there was already a value at the current index.
    pub fn push<T>(&mut self, value: &T)
    where
        T: Storable + scale::EncodeLike<V>,
    {
        let slot = self.len();
        self.set_len(slot.checked_add(1).expect("unable to checked_add"));

        assert!(self.elements.insert(slot, value).is_none());
    }

    /// Try to append an element to the back of the vector.
    ///
    /// Returns:
    ///
    /// * `Ok(())` if the value was inserted successfully
    /// * `Err(_)` if the encoded value exceeds the static buffer size.
    pub fn try_push<T>(&mut self, value: &T) -> Result<(), ink_env::Error>
    where
        T: Storable + scale::EncodeLike<V>,
    {
        let slot = self.len();
        self.set_len(slot.checked_add(1).unwrap());

        assert!(self.elements.try_insert(slot, value)?.is_none());

        Ok(())
    }

    /// Clears the last element from the storage and returns it.
    /// Shrinks the length of the vector by one.
    //
    /// Returns `None` if the vector is empty or if the last
    /// element was already cleared from storage.
    ///
    /// # Panics
    ///
    /// * If the value overgrows the static buffer size.
    pub fn pop(&mut self) -> Option<V> {
        if self.is_empty() {
            return None;
        }

        let slot = self.len().checked_sub(1).unwrap();
        self.set_len(slot);

        self.elements.take(slot)
    }

    /// Try to clear and return the last element from storage.
    /// Shrinks the length of the vector by one.
    //
    /// Returns `None` if the vector is empty.
    ///
    /// Returns
    ///
    /// `Some(Ok(_))` containing the value if it existed and was decoded successfully.
    /// `Some(Err(_))` if the value existed but its length exceeds the static buffer size.
    /// `None` if the vector is empty.
    pub fn try_pop(&mut self) -> Option<Result<V, ink_env::Error>> {
        if self.is_empty() {
            return None;
        }

        let slot = self.len().checked_sub(1).expect("unable to checked_sub");
        self.set_len(slot);

        self.elements.try_take(slot)
    }

    /// Get a copy of the last element without removing it from storage.
    ///
    /// # Panics
    ///
    /// * If the value overgrows the static buffer size.
    pub fn peek(&self) -> Option<V> {
        if self.is_empty() {
            return None;
        }

        let slot = self.len().checked_sub(1).expect("unable to checked_sub");
        self.elements.get(slot)
    }

    /// Try to get a copy of the last element without removing it from storage.
    ///
    /// Returns:
    ///
    /// `Some(Ok(_))` containing the value if it existed and was decoded successfully.
    /// `Some(Err(_))` if the value existed but its length exceeds the static buffer size.
    /// `None` if the vector is empty.
    pub fn try_peek(&self) -> Option<Result<V, ink_env::Error>> {
        if self.is_empty() {
            return None;
        }

        let slot = self.len().checked_sub(1).expect("unable to checked_sub");
        self.elements.try_get(slot)
    }

    /// Access an element at given `index`.
    ///
    /// Returns `None` if there was no value at the `index`.
    ///
    /// # Panics
    ///
    /// * If encoding the element exceeds the static buffer size.
    pub fn get(&self, index: u32) -> Option<V> {
        self.elements.get(index)
    }

    /// Try to access an element at given `index`.
    ///
    /// Returns:
    ///
    /// * `Some(Ok(_))` containing the value if it existed and was decoded successfully.
    /// * `Some(Err(_))` if the value existed but its length exceeds the static buffer
    ///   size.
    /// * `None` if there was no value at `index`.
    pub fn try_get(&self, index: u32) -> Option<ink_env::Result<V>> {
        self.elements.try_get(index)
    }

    /// Set the `value` at given `index`.
    ///
    /// # Panics
    ///
    /// * If the index is out of bounds.
    /// * If decoding the element exceeds the static buffer size.
    pub fn set<T>(&mut self, index: u32, value: &T) -> Option<u32>
    where
        T: Storable + EncodeLike<V>,
    {
        assert!(index < self.len());

        self.elements.insert(index, value)
    }

    /// Try to set the `value` at given `index`.
    ///
    /// Returns:
    ///
    /// * `Ok(Some(_))` if the value was inserted successfully, containing the size in
    ///   bytes of the pre-existing value at the specified key if any.
    /// * `Ok(None)` if the insert was successful but there was no pre-existing value.
    /// * Err([`ink_env::Error::BufferTooSmall`]) if the encoded value exceeds the static
    ///   buffer size
    /// * Err([`ink_env::Error::ReturnError`]\([`ink_env::ReturnErrorCode::KeyNotFound`]))
    ///   if the `index` is out of bounds.
    ///
    /// # Panics
    ///
    /// Panics if `index` exceeds the length of the vector.
    pub fn try_set<T>(
        &mut self,
        index: u32,
        value: &T,
    ) -> Result<Option<u32>, ink_env::Error>
    where
        T: Storable + EncodeLike<V>,
    {
        if index >= self.len() {
            return Err(ink_env::ReturnErrorCode::KeyNotFound.into());
        }

        self.elements.try_insert(index, value)
    }

    /// Delete all elements from storage.
    ///
    /// # Warning
    ///
    /// This iterates through all elements in the vector; complexity is O(n).
    /// It might not be possible to clear large vectors within a single block!
    pub fn clear(&mut self) {
        for i in 0..self.len() {
            self.elements.remove(i);
        }
        self.set_len(0);
    }

    /// Clears the value of the element at `index`. It doesn't change the length of the
    /// vector.
    ///
    /// # Panics
    ///
    /// Panics if `index` exceeds the length of the vector.
    pub fn clear_at(&mut self, index: u32) {
        assert!(index < self.len());

        self.elements.remove(index);
    }
}

impl<V, KeyType> FromIterator<V> for StorageVec<V, KeyType>
where
    V: Packed + EncodeLike<V>,
    KeyType: StorageKey,
{
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let mut result = StorageVec::<V, KeyType>::new();

        for element in iter {
            result.push(&element);
        }

        result
    }
}

impl<V, KeyType> ::core::fmt::Debug for StorageVec<V, KeyType>
where
    V: Packed,
    KeyType: StorageKey,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("StorageVec")
            .field("key", &KeyType::KEY)
            .field("len", &self.len)
            .field("len_cached", &self.len_cached)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ManualKey;

    #[test]
    fn empty_vec_works_as_expected() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut array: StorageVec<String> = StorageVec::new();

            assert_eq!(array.pop(), None);
            assert_eq!(array.peek(), None);
            assert_eq!(array.len(), 0);
            assert!(array.is_empty());

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn push_and_pop_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut array: StorageVec<String> = StorageVec::new();

            let value = "test".to_string();
            array.push(&value);
            assert_eq!(array.len(), 1);
            assert_eq!(array.pop(), Some(value));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn storage_keys_are_correct() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            const BASE: u32 = 123;
            let mut array: StorageVec<u8, ManualKey<BASE>> = StorageVec::new();

            let expected_value = 127;
            array.push(&expected_value);

            let actual_length = ink_env::get_contract_storage::<_, u32>(&BASE);
            assert_eq!(actual_length, Ok(Some(1)));

            let actual_value = ink_env::get_contract_storage::<_, u8>(&(BASE, 0u32));
            assert_eq!(actual_value, Ok(Some(expected_value)));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn push_and_pop_work_for_two_vecs_with_same_manual_key() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let expected_value = 255;

            let mut array: StorageVec<u8, ManualKey<{ u32::MIN }>> = StorageVec::new();
            array.push(&expected_value);

            let mut array2: StorageVec<u8, ManualKey<{ u32::MIN }>> = StorageVec::new();
            assert_eq!(array2.pop(), Some(expected_value));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn set_and_get_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut array: StorageVec<String> = StorageVec::new();

            let value = "test".to_string();
            array.push(&value);
            assert_eq!(array.get(0), Some(value));
            assert_eq!(array.len(), 1);

            let replaced_value = "foo".to_string();
            array.set(0, &replaced_value);
            assert_eq!(array.get(0), Some(replaced_value));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic]
    fn set_panics_on_oob() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            StorageVec::<u8>::new().set(0, &0);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn clear_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut array: StorageVec<u128> = (0..1024).collect();

            array.clear();

            assert_eq!(array.len(), 0);
            assert_eq!(array.pop(), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn clear_on_empty_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut array: StorageVec<bool> = StorageVec::new();

            array.clear();

            assert_eq!(array.len(), 0);
            assert_eq!(array.pop(), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn clear_at_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut array: StorageVec<u64> = (0..1024).collect();

            array.clear_at(0);
            assert_eq!(array.len(), 1024);
            assert_eq!(array.get(0), None);

            let last_idx = array.len() - 1;
            assert_eq!(array.get(last_idx), Some(1023));
            array.clear_at(last_idx);
            assert_eq!(array.get(last_idx), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic]
    fn clear_at_invalid_index_panics() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            StorageVec::<u32>::new().clear_at(0);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn try_get_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let array: StorageVec<u32> = (0..10).collect();

            assert_eq!(array.try_get(0), Some(Ok(0)));
            assert_eq!(array.try_get(11), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn try_set_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut array: StorageVec<u32> = (0..10).collect();

            assert_eq!(array.try_set(0, &1), Ok(Some(4)));
            assert_eq!(
                array.try_set(10, &1),
                Err(ink_env::Error::ReturnError(
                    ink_env::ReturnErrorCode::KeyNotFound
                ))
            );

            array.clear_at(0);
            assert_eq!(array.try_set(0, &1), Ok(None));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn fallible_push_pop_peek_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut array: StorageVec<u32> = (0..10).collect();

            assert_eq!(array.try_push(&10), Ok(()));
            assert_eq!(array.try_pop(), Some(Ok(10)));
            assert_eq!(array.try_peek(), Some(Ok(9)));

            array.clear();
            assert_eq!(array.try_pop(), None);
            assert_eq!(array.try_peek(), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn peek_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut array = StorageVec::<u32>::new();
            assert_eq!(array.peek(), None);

            array.push(&0);
            array.push(&9);

            assert_eq!(array.peek(), Some(9));
            assert_eq!(array.peek(), Some(9));
            assert_eq!(array.len(), 2);

            array.clear();
            assert_eq!(array.peek(), None);
            assert_eq!(array.len(), 0);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn from_iter_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let array = StorageVec::<u32>::from_iter([u32::MIN, u32::MAX]);

            assert_eq!(array.len(), 2);
            assert_eq!(array.get(0), Some(u32::MIN));
            assert_eq!(array.get(1), Some(u32::MAX));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic(
        expected = "assertion failed: cached_len.is_none() || self.len.get() == cached_len"
    )]
    fn cached_len_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let array = StorageVec::<u32>::from_iter([u32::MIN, u32::MAX]);

            assert_eq!(array.len(), 2);

            // Force overwrite the length
            Lazy::<u32>::new().set(&u32::MAX);

            // This should fail the debug assert
            let _ = array.len();

            Ok(())
        })
        .unwrap()
    }
}
