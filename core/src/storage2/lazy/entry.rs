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

use crate::storage2::{
    traits2::{
        clear_spread_root_opt,
        pull_spread_root_opt,
        push_spread_root_opt,
        KeyPtr as KeyPtr2,
        SpreadLayout,
    },
    ClearForward,
    KeyPtr,
    PushForward,
    StorageFootprint,
};
use core::cell::Cell;

/// The entry of a single cached value of a lazy storage data structure.
#[derive(Debug, Clone)]
pub struct Entry<T> {
    /// The value or `None` if the value has been removed.
    value: Option<T>,
    /// This is `true` if the `value` is dirty and needs to be synchronized
    /// with the underlying contract storage.
    state: Cell<EntryState>,
}

/// The state of the entry.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

    fn pull_spread(ptr: &mut KeyPtr2) -> Self {
        let root_key = ptr.next_for::<Self>();
        Self::new(pull_spread_root_opt::<T>(&root_key), EntryState::Preserved)
    }

    fn push_spread(&self, ptr: &mut KeyPtr2) {
        if !self.is_mutated() {
            return
        }
        self.state.set(EntryState::Preserved);
        let root_key = ptr.next_for::<Self>();
        push_spread_root_opt::<T>(self.value().into(), &root_key);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr2) {
        let root_key = ptr.next_for::<Self>();
        clear_spread_root_opt::<T>(self.value().into(), &root_key);
    }
}

impl<T> PushForward for Entry<T>
where
    T: PushForward + StorageFootprint,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        if self.state.get() != EntryState::Mutated {
            return
        }
        // Reset the state because we just synced.
        self.state.set(EntryState::Preserved);
        // Since `self.value` is of type `Option` this will eventually
        // clear the underlying storage entry if `self.value` is `None`.
        self.value.push_forward(ptr);
    }
}

impl<T> ClearForward for Entry<T>
where
    T: ClearForward + StorageFootprint,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        // Reset the state because we just synced.
        self.state.set(EntryState::Preserved);
        // Since `self.value` is of type `Option` this will eventually
        // clear the underlying storage entry if `self.value` is `None`.
        self.value.clear_forward(ptr);
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
        self.state.set(
            if self.value.is_some() {
                EntryState::Mutated
            } else {
                EntryState::Preserved
            },
        );
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
