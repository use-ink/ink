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

use super::{
    CacheCell,
    EntryState,
    StorageEntry,
};
use crate::traits::{
    clear_spread_root_opt,
    pull_spread_root_opt,
    ExtKeyPtr,
    KeyPtr,
    SpreadLayout,
};
use core::{
    fmt,
    fmt::Debug,
    ptr::NonNull,
};
use ink_primitives::Key;

/// A lazy storage entity.
///
/// This loads its value from storage upon first use.
///
/// # Note
///
/// Use this if the storage field doesn't need to be loaded in some or most cases.
pub struct LazyCell<T>
where
    T: SpreadLayout,
{
    /// The key to lazily load the value from.
    ///
    /// # Note
    ///
    /// This can be `None` on contract initialization where a `LazyCell` is
    /// normally initialized given a concrete value.
    key: Option<Key>,
    /// The low-level cache for the lazily loaded storage value.
    ///
    /// # Safety (Dev)
    ///
    /// We use `UnsafeCell` instead of `RefCell` because
    /// the intended use-case is to hand out references (`&` and `&mut`)
    /// to the callers of `Lazy`. This cannot be done without `unsafe`
    /// code even with `RefCell`. Also `RefCell` has a larger memory footprint
    /// and has additional overhead that we can avoid by the interface
    /// and the fact that ink! code is always run single-threaded.
    /// Being efficient is important here because this is intended to be
    /// a low-level primitive with lots of dependencies.
    cache: CacheCell<Option<StorageEntry<T>>>,
}

impl<T> Debug for LazyCell<T>
where
    T: Debug + SpreadLayout,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LazyCell")
            .field("key", &self.key)
            .field("cache", self.cache.as_inner())
            .finish()
    }
}

#[test]
fn debug_impl_works() -> ink_env::Result<()> {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let c1 = <LazyCell<i32>>::new(None);
        assert_eq!(
            format!("{:?}", &c1),
            "LazyCell { key: None, cache: Some(Entry { value: None, state: Mutated }) }",
        );
        let c2 = <LazyCell<i32>>::new(Some(42));
        assert_eq!(
            format!("{:?}", &c2),
            "LazyCell { key: None, cache: Some(Entry { value: Some(42), state: Mutated }) }",
        );
        let c3 = <LazyCell<i32>>::lazy(Key::from([0x00; 32]));
        assert_eq!(
            format!("{:?}", &c3),
            "LazyCell { \
            key: Some(Key(0x_\
                0000000000000000_\
                0000000000000000_\
                0000000000000000_\
                0000000000000000)\
            ), \
            cache: None \
        }",
        );
        Ok(())
    })
}

impl<T> Drop for LazyCell<T>
where
    T: SpreadLayout,
{
    fn drop(&mut self) {
        if let Some(root_key) = self.key() {
            match self.entry() {
                Some(entry) => {
                    // The inner cell needs to be cleared, no matter if it has
                    // been loaded or not. Otherwise there might be leftovers.
                    // Load from storage and then clear:
                    clear_spread_root_opt::<T, _>(root_key, || entry.value().into())
                }
                None => {
                    // The value is not yet in the cache. we need it in there
                    // though in order to properly clean up.
                    if <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP {
                        // The inner cell needs to be cleared, no matter if it has
                        // been loaded or not. Otherwise there might be leftovers.
                        // Load from storage and then clear:
                        clear_spread_root_opt::<T, _>(root_key, || self.get())
                    } else {
                        // Clear without loading from storage:
                        let footprint = <T as SpreadLayout>::FOOTPRINT;
                        assert_footprint_threshold(footprint);
                        let mut key_ptr = KeyPtr::from(*root_key);
                        for _ in 0..footprint {
                            ink_env::clear_contract_storage(key_ptr.advance_by(1));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::Layout;

    impl<T> StorageLayout for LazyCell<T>
    where
        T: StorageLayout + SpreadLayout,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            <T as StorageLayout>::layout(key_ptr)
        }
    }
};

impl<T> SpreadLayout for LazyCell<T>
where
    T: SpreadLayout,
{
    const FOOTPRINT: u64 = <T as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        let root_key = ExtKeyPtr::next_for::<Self>(ptr);
        Self::lazy(*root_key)
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        let root_key = ExtKeyPtr::next_for::<Self>(ptr);
        if let Some(entry) = self.entry() {
            entry.push_spread_root(root_key)
        }
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        let root_key = ExtKeyPtr::next_for::<Self>(ptr);
        match <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP {
            true => {
                // The inner cell needs to be cleared, no matter if it has
                // been loaded or not. Otherwise there might be leftovers.
                // Load from storage and then clear:
                clear_spread_root_opt::<T, _>(root_key, || self.get())
            }
            false => {
                // Clear without loading from storage:
                let footprint = <T as SpreadLayout>::FOOTPRINT;
                assert_footprint_threshold(footprint);
                let mut key_ptr = KeyPtr::from(*root_key);
                for _ in 0..footprint {
                    ink_env::clear_contract_storage(key_ptr.advance_by(1));
                }
            }
        }
    }
}

// # Developer Note
//
// Implementing PackedLayout for LazyCell is not useful since that would
// potentially allow overlapping distinct LazyCell instances by pulling
// from the same underlying storage cell.
//
// If a user wants a packed LazyCell they can instead pack its inner type.

impl<T> From<T> for LazyCell<T>
where
    T: SpreadLayout,
{
    fn from(value: T) -> Self {
        Self::new(Some(value))
    }
}

impl<T> Default for LazyCell<T>
where
    T: Default + SpreadLayout,
{
    fn default() -> Self {
        Self::new(Some(Default::default()))
    }
}

impl<T> LazyCell<T>
where
    T: SpreadLayout,
{
    /// Creates an already populated lazy storage cell.
    ///
    /// # Note
    ///
    /// Since this already has a value it will never actually load from
    /// the contract storage.
    #[must_use]
    pub fn new(value: Option<T>) -> Self {
        Self {
            key: None,
            cache: CacheCell::new(Some(StorageEntry::new(value, EntryState::Mutated))),
        }
    }

    /// Creates a lazy storage cell for the given key.
    ///
    /// # Note
    ///
    /// This will actually lazily load from the associated storage cell
    /// upon access.
    #[must_use]
    pub fn lazy(key: Key) -> Self {
        Self {
            key: Some(key),
            cache: CacheCell::new(None),
        }
    }

    /// Returns the lazy key if any.
    ///
    /// # Note
    ///
    /// The key is `None` if the `LazyCell` has been initialized as a value.
    /// This generally only happens in ink! constructors.
    fn key(&self) -> Option<&Key> {
        self.key.as_ref()
    }

    /// Returns the cached entry.
    fn entry(&self) -> Option<&StorageEntry<T>> {
        self.cache.as_inner().as_ref()
    }
}

impl<T> LazyCell<T>
where
    T: SpreadLayout,
{
    /// Loads the storage entry.
    ///
    /// Tries to load the entry from cache and falls back to lazily load the
    /// entry from the contract storage.
    unsafe fn load_through_cache(&self) -> NonNull<StorageEntry<T>> {
        // SAFETY: This is critical because we mutably access the entry.
        //         However, we mutate the entry only if it is vacant.
        //         If the entry is occupied by a value we return early.
        //         This way we do not invalidate pointers to this value.
        let cache = &mut *self.cache.get_ptr().as_ptr();
        if cache.is_none() {
            // Load value from storage and then return the cached entry.
            let value = self
                .key
                .map(|key| pull_spread_root_opt::<T>(&key))
                .unwrap_or(None);
            *cache = Some(StorageEntry::new(value, EntryState::Preserved));
        }
        debug_assert!(cache.is_some());
        NonNull::from(cache.as_mut().expect("unpopulated cache entry"))
    }

    /// Returns a shared reference to the entry.
    fn load_entry(&self) -> &StorageEntry<T> {
        // SAFETY: We load the entry either from cache of from contract storage.
        //
        //         This is safe because we are just returning a shared reference
        //         from within a `&self` method. This also cannot change the
        //         loaded value and thus cannot change the `mutate` flag of the
        //         entry. Aliases using this method are safe since ink! is
        //         single-threaded.
        unsafe { &*self.load_through_cache().as_ptr() }
    }

    /// Returns an exclusive reference to the entry.
    fn load_entry_mut(&mut self) -> &mut StorageEntry<T> {
        // SAFETY: We load the entry either from cache of from contract storage.
        //
        //         This is safe because we are just returning an exclusive reference
        //         from within a `&mut self` method. This may change the
        //         loaded value and thus the `mutate` flag of the entry is set.
        //         Aliases cannot happen through this method since ink! is
        //         single-threaded.
        let entry = unsafe { &mut *self.load_through_cache().as_ptr() };
        entry.replace_state(EntryState::Mutated);
        entry
    }

    /// Returns a shared reference to the value.
    ///
    /// # Note
    ///
    /// This eventually lazily loads the value from the contract storage.
    ///
    /// # Panics
    ///
    /// If decoding the loaded value to `T` failed.
    #[must_use]
    pub fn get(&self) -> Option<&T> {
        self.load_entry().value().into()
    }

    /// Returns an exclusive reference to the value.
    ///
    /// # Note
    ///
    /// This eventually lazily loads the value from the contract storage.
    ///
    /// # Panics
    ///
    /// If decoding the loaded value to `T` failed.
    #[must_use]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.load_entry_mut().value_mut().into()
    }

    /// Sets the value in this cell to `value`, without executing any reads.
    ///
    /// # Note
    ///
    /// No reads from contract storage will be executed.
    ///
    /// This method should be preferred over dereferencing or `get_mut`
    /// in case the returned value is of no interest to the caller.
    ///
    /// # Panics
    ///
    /// If accessing the inner value fails.
    #[inline]
    pub fn set(&mut self, new_value: T) {
        // SAFETY: This is critical because we mutably access the entry.
        let cache = unsafe { &mut *self.cache.get_ptr().as_ptr() };
        if let Some(cache) = cache.as_mut() {
            //  Cache is already populated we simply overwrite its already existing value.
            cache.put(Some(new_value));
        } else {
            // Cache is empty, so we simply set the cache to the value.
            // The key does not need to exist for this to work, we only need to
            // write the value into the cache and are done. Writing to contract
            // storage happens during setup/teardown of a contract.
            *cache = Some(StorageEntry::new(Some(new_value), EntryState::Mutated));
        }
        debug_assert!(cache.is_some());
    }
}

/// Asserts that the given `footprint` is below `FOOTPRINT_CLEANUP_THRESHOLD`.
fn assert_footprint_threshold(footprint: u64) {
    let footprint_threshold = crate::traits::FOOTPRINT_CLEANUP_THRESHOLD;
    assert!(
        footprint <= footprint_threshold,
        "cannot clean-up a storage entity with a footprint of {}. maximum threshold for clean-up is {}.",
        footprint,
        footprint_threshold,
    );
}

#[cfg(test)]
mod tests {
    use super::{
        EntryState,
        LazyCell,
        StorageEntry,
    };
    use crate::{
        traits::{
            KeyPtr,
            SpreadLayout,
        },
        Lazy,
    };
    use ink_env::test::run_test;
    use ink_primitives::Key;

    #[test]
    fn new_works() {
        // Initialized via some value:
        let mut a = <LazyCell<u8>>::new(Some(b'A'));
        assert_eq!(a.key(), None);
        assert_eq!(
            a.entry(),
            Some(&StorageEntry::new(Some(b'A'), EntryState::Mutated))
        );
        assert_eq!(a.get(), Some(&b'A'));
        assert_eq!(a.get_mut(), Some(&mut b'A'));
        // Initialized as none:
        let mut b = <LazyCell<u8>>::new(None);
        assert_eq!(b.key(), None);
        assert_eq!(
            b.entry(),
            Some(&StorageEntry::new(None, EntryState::Mutated))
        );
        assert_eq!(b.get(), None);
        assert_eq!(b.get_mut(), None);
        // Same as default or from:
        let default_lc = <LazyCell<u8>>::default();
        let from_lc = LazyCell::from(u8::default());
        let new_lc = LazyCell::new(Some(u8::default()));
        assert_eq!(default_lc.get(), from_lc.get());
        assert_eq!(from_lc.get(), new_lc.get());
        assert_eq!(new_lc.get(), Some(&u8::default()));
    }

    #[test]
    fn lazy_works() -> ink_env::Result<()> {
        run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let root_key = Key::from([0x42; 32]);
            let cell = <LazyCell<u8>>::lazy(root_key);
            assert_eq!(cell.key(), Some(&root_key));
            Ok(())
        })
    }

    #[test]
    fn lazy_get_works() -> ink_env::Result<()> {
        run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let cell = <LazyCell<u8>>::lazy(Key::from([0x42; 32]));
            let value = cell.get();
            // We do the normally unreachable check in order to have an easier
            // time finding the issue if the above execution did not panic.
            assert_eq!(value, None);
            Ok(())
        })
    }

    #[test]
    fn get_mut_works() {
        let mut cell = <LazyCell<i32>>::new(Some(1));
        assert_eq!(cell.get(), Some(&1));
        *cell.get_mut().unwrap() += 1;
        assert_eq!(cell.get(), Some(&2));
    }

    #[test]
    fn spread_layout_works() -> ink_env::Result<()> {
        run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let cell_a0 = <LazyCell<u8>>::new(Some(b'A'));
            assert_eq!(cell_a0.get(), Some(&b'A'));
            // Push `cell_a0` to the contract storage.
            // Then, pull `cell_a1` from the contract storage and check if it is
            // equal to `cell_a0`.
            let root_key = Key::from([0x42; 32]);
            SpreadLayout::push_spread(&cell_a0, &mut KeyPtr::from(root_key));
            let cell_a1 =
                <LazyCell<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
            assert_eq!(cell_a1.get(), cell_a0.get());
            assert_eq!(cell_a1.get(), Some(&b'A'));
            assert_eq!(
                cell_a1.entry(),
                Some(&StorageEntry::new(Some(b'A'), EntryState::Preserved))
            );
            // Also test if a lazily instantiated cell works:
            let cell_a2 = <LazyCell<u8>>::lazy(root_key);
            assert_eq!(cell_a2.get(), cell_a0.get());
            assert_eq!(cell_a2.get(), Some(&b'A'));
            assert_eq!(
                cell_a2.entry(),
                Some(&StorageEntry::new(Some(b'A'), EntryState::Preserved))
            );
            // Test if clearing works:
            SpreadLayout::clear_spread(&cell_a1, &mut KeyPtr::from(root_key));
            let cell_a3 = <LazyCell<u8>>::lazy(root_key);
            assert_eq!(cell_a3.get(), None);
            assert_eq!(
                cell_a3.entry(),
                Some(&StorageEntry::new(None, EntryState::Preserved))
            );
            Ok(())
        })
    }

    #[test]
    fn set_works() {
        let mut cell = <LazyCell<i32>>::new(Some(1));
        cell.set(23);
        assert_eq!(cell.get(), Some(&23));
    }

    #[test]
    fn lazy_set_works() -> ink_env::Result<()> {
        run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut cell = <LazyCell<u8>>::lazy(Key::from([0x42; 32]));
            let value = cell.get();
            assert_eq!(value, None);

            cell.set(13);
            assert_eq!(cell.get(), Some(&13));
            Ok(())
        })
    }

    #[test]
    fn lazy_set_works_with_spread_layout_push_pull() -> ink_env::Result<()> {
        run_test::<ink_env::DefaultEnvironment, _>(|_| {
            type MaybeValue = Option<u8>;

            // Initialize a LazyCell with None and push it to `k`
            let k = Key::from([0x00; 32]);
            let val: MaybeValue = None;
            SpreadLayout::push_spread(&Lazy::new(val), &mut KeyPtr::from(k));

            // Pull another instance `v` from `k`, check that it is `None`
            let mut v =
                <Lazy<MaybeValue> as SpreadLayout>::pull_spread(&mut KeyPtr::from(k));
            assert_eq!(*v, None);

            // Set `v` using `set` to an actual value
            let actual_value: MaybeValue = Some(13);
            Lazy::set(&mut v, actual_value);

            // Push `v` to `k`
            SpreadLayout::push_spread(&v, &mut KeyPtr::from(k));

            // Load `v2` from `k`
            let v2 =
                <Lazy<MaybeValue> as SpreadLayout>::pull_spread(&mut KeyPtr::from(k));

            // Check that V2 is the set value
            assert_eq!(*v2, Some(13));

            Ok(())
        })
    }

    #[test]
    fn regression_test_for_issue_528() -> ink_env::Result<()> {
        run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let root_key = Key::from([0x00; 32]);
            {
                // Step 1: Push a valid pair onto the contract storage.
                let pair = (LazyCell::new(Some(1i32)), 2i32);
                SpreadLayout::push_spread(&pair, &mut KeyPtr::from(root_key));
            }
            {
                // Step 2: Pull the pair from the step before.
                //
                // 1. Change the second `i32` value of the pair.
                // 2. Push the pair again to contract storage.
                //
                // We prevent the intermediate instance from clearing the storage preemtively by wrapping
                // it inside `ManuallyDrop`. The third step will clean up the same storage region afterwards.
                //
                // We explicitly do not touch or assert the value of `pulled_pair.0` in order to trigger
                // the bug.
                let pulled_pair: (LazyCell<i32>, i32) =
                    SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
                let mut pulled_pair = core::mem::ManuallyDrop::new(pulled_pair);
                assert_eq!(pulled_pair.1, 2i32);
                pulled_pair.1 = 3i32;
                SpreadLayout::push_spread(&*pulled_pair, &mut KeyPtr::from(root_key));
            }
            {
                // Step 3: Pull the pair again from the storage.
                //
                // If the bug with `Lazy` that has been fixed in PR #528 has been fixed we should be
                // able to inspect the correct values for both pair entries which is: `(Some(1), 3)`
                let pulled_pair: (LazyCell<i32>, i32) =
                    SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
                assert_eq!(pulled_pair.0.get(), Some(&1i32));
                assert_eq!(pulled_pair.1, 3i32);
            }
            Ok(())
        })
    }

    #[test]
    fn regression_test_for_issue_570() -> ink_env::Result<()> {
        run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let root_key = Key::from([0x00; 32]);
            {
                // Step 1: Push two valid values one after the other to contract storage.
                // The first value needs to be an `Option::None` value, since the bug was
                // then messing up following pointers.
                let v1: Option<u32> = None;
                let v2: u32 = 13;
                let mut ptr = KeyPtr::from(root_key);

                SpreadLayout::push_spread(&v1, &mut ptr);
                SpreadLayout::push_spread(&v2, &mut ptr);
            }
            {
                // Step 2: Pull the values from the step before.
                //
                // 1. Change the first values `None` to `Some(...)`.
                // 2. Push the first value again to contract storage.
                //
                // We prevent the intermediate instance from clearing the storage preemptively
                // by wrapping it inside `ManuallyDrop`. The third step will clean up the same
                // storage region afterwards.
                let mut ptr = KeyPtr::from(root_key);
                let pulled_v1: Option<u32> = SpreadLayout::pull_spread(&mut ptr);
                let mut pulled_v1 = core::mem::ManuallyDrop::new(pulled_v1);

                let pulled_v2: u32 = SpreadLayout::pull_spread(&mut ptr);
                let pulled_v2 = core::mem::ManuallyDrop::new(pulled_v2);

                assert_eq!(*pulled_v1, None);
                assert_eq!(*pulled_v2, 13);

                *pulled_v1 = Some(99u32);
                SpreadLayout::push_spread(&*pulled_v1, &mut KeyPtr::from(root_key));
            }
            {
                // Step 3: Pull the values again from the storage.
                //
                // If the bug with `Option` has been fixed in PR #520 we must be able to inspect
                // the correct values for both entries.
                let mut ptr = KeyPtr::from(root_key);
                let pulled_v1: Option<u32> = SpreadLayout::pull_spread(&mut ptr);
                let pulled_v2: u32 = SpreadLayout::pull_spread(&mut ptr);

                assert_eq!(pulled_v1, Some(99));
                assert_eq!(pulled_v2, 13);
            }
            Ok(())
        })
    }

    #[test]
    fn second_regression_test_for_issue_570() -> ink_env::Result<()> {
        run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // given
            let root_key = Key::from([0x00; 32]);
            let none: Option<u32> = None;
            let some: Option<u32> = Some(13);

            // when
            let mut ptr_push_none = KeyPtr::from(root_key);
            SpreadLayout::push_spread(&none, &mut ptr_push_none);
            let mut ptr_pull_none = KeyPtr::from(root_key);
            let v1: Option<u32> = SpreadLayout::pull_spread(&mut ptr_pull_none);
            assert!(v1.is_none());
            let mut ptr_clear_none = KeyPtr::from(root_key);
            SpreadLayout::clear_spread(&none, &mut ptr_clear_none);

            let mut ptr_push_some = KeyPtr::from(root_key);
            SpreadLayout::push_spread(&some, &mut ptr_push_some);
            let mut ptr_pull_some = KeyPtr::from(root_key);
            let v2: Option<u32> = SpreadLayout::pull_spread(&mut ptr_pull_some);
            assert!(v2.is_some());
            let mut ptr_clear_some = KeyPtr::from(root_key);
            SpreadLayout::clear_spread(&some, &mut ptr_clear_some);

            // then
            // the bug which we observed was that the pointer after push/pull/clear
            // was set so a different value if the `Option` was `None` vs. if it was
            // `Some`.
            //
            // if the bug has been fixed the pointer must be the same for `None`
            // and `Some` after push/pull/clear. otherwise subsequent operations using
            // the pointer will break as soon as the `Option` is changed to it's
            // opposite (`None` -> `Some`, `Some` -> `None`).
            let mut expected_post_op_ptr = KeyPtr::from(root_key);
            // advance one time after the cell containing `self.is_some() as u8` has been read
            expected_post_op_ptr.advance_by(1);
            // advance another time after the cell containing the inner `Option` value
            // has either been skipped (in case of the previous cell being `None`) or
            // read (in case of `Some`).
            expected_post_op_ptr.advance_by(1);

            assert_eq!(expected_post_op_ptr, ptr_push_none);
            assert_eq!(ptr_push_none, ptr_push_some);

            assert_eq!(expected_post_op_ptr, ptr_pull_none);
            assert_eq!(ptr_pull_none, ptr_pull_some);

            assert_eq!(expected_post_op_ptr, ptr_clear_none);
            assert_eq!(ptr_clear_none, ptr_clear_some);

            Ok(())
        })
    }

    #[test]
    #[should_panic(expected = "encountered empty storage cell")]
    fn nested_lazies_are_cleared_completely_after_pull() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // given
            let root_key = Key::from([0x42; 32]);
            let nested_lazy: Lazy<Lazy<u32>> = Lazy::new(Lazy::new(13u32));
            SpreadLayout::push_spread(&nested_lazy, &mut KeyPtr::from(root_key));
            let pulled_lazy = <Lazy<Lazy<u32>> as SpreadLayout>::pull_spread(
                &mut KeyPtr::from(root_key),
            );

            // when
            SpreadLayout::clear_spread(&pulled_lazy, &mut KeyPtr::from(root_key));

            // then
            let contract_id = ink_env::test::get_current_contract_account_id::<
                ink_env::DefaultEnvironment,
            >()
            .expect("Cannot get contract id");
            let used_cells = ink_env::test::count_used_storage_cells::<
                ink_env::DefaultEnvironment,
            >(&contract_id)
            .expect("used cells must be returned");
            assert_eq!(used_cells, 0);
            let _ = *<Lazy<Lazy<u32>> as SpreadLayout>::pull_spread(&mut KeyPtr::from(
                root_key,
            ));
            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic(expected = "encountered empty storage cell")]
    fn lazy_drop_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // given
            let root_key = Key::from([0x42; 32]);

            // when
            let setup_result = std::panic::catch_unwind(|| {
                let lazy: Lazy<u32> = Lazy::new(13u32);
                SpreadLayout::push_spread(&lazy, &mut KeyPtr::from(root_key));
                let _pulled_lazy =
                    <Lazy<u32> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
                // lazy is dropped which should clear the cells
            });
            assert!(setup_result.is_ok(), "setup should not panic");

            // then
            let contract_id = ink_env::test::get_current_contract_account_id::<
                ink_env::DefaultEnvironment,
            >()
            .expect("Cannot get contract id");
            let used_cells = ink_env::test::count_used_storage_cells::<
                ink_env::DefaultEnvironment,
            >(&contract_id)
            .expect("used cells must be returned");
            assert_eq!(used_cells, 0);
            let _ =
                *<Lazy<u32> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic(expected = "encountered empty storage cell")]
    fn lazy_drop_works_with_greater_footprint() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // given
            let root_key = Key::from([0x42; 32]);

            // when
            let setup_result = std::panic::catch_unwind(|| {
                let lazy: Lazy<[u32; 5]> = Lazy::new([13, 14, 15, 16, 17]);
                SpreadLayout::push_spread(&lazy, &mut KeyPtr::from(root_key));
                let _pulled_lazy = <Lazy<[u32; 5]> as SpreadLayout>::pull_spread(
                    &mut KeyPtr::from(root_key),
                );
                // lazy is dropped which should clear the cells
            });
            assert!(setup_result.is_ok(), "setup should not panic");

            // then
            let contract_id = ink_env::test::get_current_contract_account_id::<
                ink_env::DefaultEnvironment,
            >()
            .expect("Cannot get contract id");
            let used_cells = ink_env::test::count_used_storage_cells::<
                ink_env::DefaultEnvironment,
            >(&contract_id)
            .expect("used cells must be returned");
            assert_eq!(used_cells, 0);
            let _ =
                *<Lazy<u32> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
            Ok(())
        })
        .unwrap()
    }
}
