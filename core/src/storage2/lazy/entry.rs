// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

#[cfg(doc)]
use crate::storage2::lazy::{
    LazyArray,
    LazyIndexMap,
};
use crate::storage2::traits::{
    clear_packed_root,
    clear_spread_root_opt,
    pull_packed_root_opt,
    pull_spread_root_opt,
    push_packed_root_opt,
    push_spread_root_opt,
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use core::{
    cell::Cell,
    fmt,
    fmt::Debug,
};
use ink_prelude::vec::Vec;
use ink_primitives::Key;

/// The entry of a single cached value of a lazy storage data structure.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry<T> {
    /// The value or `None` if the value has been removed.
    value: Option<T>,
    /// This is `true` if the `value` is dirty and needs to be synchronized
    /// with the underlying contract storage.
    state: Cell<EntryState>,
}

impl<T> Debug for Entry<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Entry")
            .field("value", &self.value)
            .field("state", &self.state.get())
            .finish()
    }
}

#[test]
fn debug_impl_works() {
    let e1 = <Entry<i32>>::new(None, EntryState::Preserved);
    assert_eq!(
        format!("{:?}", &e1),
        "Entry { value: None, state: Preserved }",
    );
    let e2 = Entry::new(Some(42), EntryState::Mutated);
    assert_eq!(
        format!("{:?}", &e2),
        "Entry { value: Some(42), state: Mutated }",
    );
}

/// The state of the entry.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryState {
    /// The entry's value must be synchronized with the contract storage.
    Mutated,
    /// The entry's value preserved the value from the contract storage.
    Preserved,
}

impl EntryState {
    /// Returns `true` if the entry state is mutated.
    pub fn is_mutated(self) -> bool {
        match self {
            EntryState::Mutated => true,
            EntryState::Preserved => false,
        }
    }

    /// Returns `true` if the entry state is preserved.
    pub fn is_preserved(self) -> bool {
        !self.is_mutated()
    }
}

impl<T> SpreadLayout for Entry<T>
where
    T: SpreadLayout,
{
    const FOOTPRINT: u64 = <T as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        let root_key = ptr.next_for::<Self>();
        Self::new(pull_spread_root_opt::<T>(&root_key), EntryState::Preserved)
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        if !self.is_mutated() {
            return
        }
        self.state.set(EntryState::Preserved);
        let root_key = ptr.next_for::<Self>();
        push_spread_root_opt::<T>(self.value().into(), &root_key);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        let root_key = ptr.next_for::<Self>();
        clear_spread_root_opt::<T>(self.value().into(), &root_key);
    }
}

impl<T> scale::Encode for Entry<T>
where
    T: scale::Encode,
{
    #[inline]
    fn size_hint(&self) -> usize {
        <Option<T> as scale::Encode>::size_hint(&self.value)
    }

    #[inline]
    fn encode_to<O: scale::Output>(&self, dest: &mut O) {
        <Option<T> as scale::Encode>::encode_to(&self.value, dest)
    }

    #[inline]
    fn encode(&self) -> Vec<u8> {
        <Option<T> as scale::Encode>::encode(&self.value)
    }

    #[inline]
    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        <Option<T> as scale::Encode>::using_encoded(&self.value, f)
    }
}

impl<T> scale::Decode for Entry<T>
where
    T: scale::Decode,
{
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        Ok(Self::new(
            <Option<T> as scale::Decode>::decode(input)?,
            EntryState::Preserved,
        ))
    }
}

impl<T> PackedLayout for Entry<T>
where
    T: PackedLayout,
{
    #[inline]
    fn pull_packed(&mut self, at: &Key) {
        PackedLayout::pull_packed(&mut self.value, at)
    }

    #[inline]
    fn push_packed(&self, at: &Key) {
        PackedLayout::push_packed(&self.value, at)
    }

    #[inline]
    fn clear_packed(&self, at: &Key) {
        PackedLayout::clear_packed(&self.value, at)
    }
}

impl<T> Entry<T>
where
    T: PackedLayout,
{
    /// Pulls the entity from the underlying associated storage as packed representation.
    ///
    /// # Note
    ///
    /// Mainly used by lazy storage abstractions that only allow operating on
    /// packed storage entities such as [`LazyIndexMap`] or [`LazyArray`].
    pub fn pull_packed_root(root_key: &Key) -> Self {
        let mut entry =
            Self::new(pull_packed_root_opt::<T>(root_key), EntryState::Preserved);
        PackedLayout::pull_packed(&mut entry, root_key);
        entry
    }

    /// Pushes the underlying associated storage as packed representation.
    ///
    /// # Note
    ///
    /// Mainly used by lazy storage abstractions that only allow operating on
    /// packed storage entities such as [`LazyIndexMap`] or [`LazyArray`].
    pub fn push_packed_root(&self, root_key: &Key) {
        if !self.is_mutated() {
            return
        }
        self.state.set(EntryState::Preserved);
        push_packed_root_opt::<T>(self.value().into(), &root_key);
    }

    /// Clears the underlying associated storage as packed representation.
    ///
    /// # Note
    ///
    /// Mainly used by lazy storage abstractions that only allow operating on
    /// packed storage entities such as [`LazyIndexMap`] or [`LazyArray`].
    pub fn clear_packed_root(&self, root_key: &Key) {
        clear_packed_root::<Option<T>>(self.value(), &root_key);
    }
}

impl<T> Entry<T> {
    /// Creates a new entry with the value and state.
    pub fn new(value: Option<T>, state: EntryState) -> Self {
        Self {
            value,
            state: Cell::new(state),
        }
    }

    /// Replaces the current entry state with the new state and returns it.
    pub fn replace_state(&mut self, new_state: EntryState) -> EntryState {
        self.state.replace(new_state)
    }

    /// Sets the entry state to the new state.
    pub fn set_state(&mut self, new_state: EntryState) {
        self.state.set(new_state);
    }

    /// Returns `true` if the cached value of the entry has potentially been mutated.
    pub fn is_mutated(&self) -> bool {
        self.state.get() == EntryState::Mutated
    }

    /// Returns a shared reference to the value of the entry.
    pub fn value(&self) -> &Option<T> {
        &self.value
    }

    /// Returns an exclusive reference to the entry value.
    ///
    /// # Note
    ///
    /// This changes the `mutate` state of the entry if the entry was occupied
    /// since the caller could potentially change the returned value.
    pub fn value_mut(&mut self) -> &mut Option<T> {
        if self.value.is_some() {
            self.state.set(EntryState::Mutated);
        }
        &mut self.value
    }

    /// Takes the value from the entry and returns it.
    ///
    /// # Note
    ///
    /// This changes the `mutate` state of the entry if the entry was occupied.
    pub fn take_value(&mut self) -> Option<T> {
        self.state.set(
            if self.value.is_some() {
                EntryState::Mutated
            } else {
                EntryState::Preserved
            },
        );
        self.value.take()
    }

    /// Converts the entry into its value.
    pub fn into_value(self) -> Option<T> {
        self.value
    }

    /// Puts the new value into the entry and returns the old value.
    ///
    /// # Note
    ///
    /// This changes the `mutate` state of the entry to `true` as long as at
    /// least one of `old_value` and `new_value` is `Some`.
    pub fn put(&mut self, new_value: Option<T>) -> Option<T> {
        match new_value {
            Some(new_value) => {
                self.state.set(EntryState::Mutated);
                self.value.replace(new_value)
            }
            None => self.take_value(),
        }
    }
}
