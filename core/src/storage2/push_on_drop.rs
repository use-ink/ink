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

use crate::storage2::traits::{
    push_spread_root,
    SpreadLayout,
};
use core::{
    mem::ManuallyDrop,
    ops::{
        Deref,
        DerefMut,
    },
};
use ink_primitives::Key;

/// Wraps a storage entity to make it push its contents upon being dropped.
///
/// This will cause the inner `T` to not drop upon the outer `PushOnDrop`
/// being dropped preventing the clearing of the associated storage region of
/// the inner value and instead pushing its current state to the contract
/// storage.
///
/// # Note
///
/// Note that this might potentially cause memory leaks if used incorrectly!
///
/// # Developer Note
///
/// In ink! we have two different and unrelated use cases for this:
///
/// 1. In Wasm compilation we wrap the dynamic storage allocator in a
///    `PushOnDrop` in order to automatically push its state to the contract
///    storage upon the end of the smart contract execution. This leaks memory
///    which isn't bad because the host side clears it up for us after contract
///    execution ends.
///    Note that we never wrap the dynamic storage allocator in a `PushOnDrop`
///    in no-Wasm (e.g. `std`) based compiles to make our off-chain test suite
///    work properly. There we need to clean-up dynamic storage allocator state
///    in order to not share state between tests.
/// 2. The ink! codegen wraps the static contract storage (`#[ink(storage)]`)
///    in a `PushOnDrop` in order to make it autmatically push its state to the
///    contract upon the end of the smart contract execution. This might leak
///    memory but it isn't too bad because the host side immediately clears it
///    up for us after the contract execution ends.
///
/// We provide this abstract utility type in case ink! users find other use
/// cases where this abstraction fits.
#[derive(Debug, PartialEq, Eq)]
pub struct PushOnDrop<T>
where
    T: SpreadLayout,
{
    /// Where to push the storage entity upon `drop`.
    at: Key,
    /// Which storage entity to push upon `drop`.
    ///
    /// The storage entity is wrapped inside a `ManuallyDrop` to avoid calling
    /// its destructor upon `drop`. The reason is that calling its destructor
    /// would generally perform a clean-up.
    what: ManuallyDrop<T>,
}

impl<T> Drop for PushOnDrop<T>
where
    T: SpreadLayout,
{
    fn drop(&mut self) {
        push_spread_root::<T>(Self::as_inner(self), &self.at)
    }
}

impl<T> PushOnDrop<T>
where
    T: SpreadLayout,
{
    /// Creates a new `PushOnDrop` wrapper.
    pub fn new(at: Key, what: T) -> Self {
        Self {
            at,
            what: ManuallyDrop::new(what),
        }
    }

    /// Returns a shared reference to the contained value.
    fn as_inner(this: &Self) -> &T {
        <ManuallyDrop<T> as Deref>::deref(&this.what)
    }

    /// Returns an exclusive reference to the contained value.
    fn as_inner_mut(this: &mut Self) -> &mut T {
        <ManuallyDrop<T> as DerefMut>::deref_mut(&mut this.what)
    }
}

impl<T> Deref for PushOnDrop<T>
where
    T: SpreadLayout,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::as_inner(self)
    }
}

impl<T> DerefMut for PushOnDrop<T>
where
    T: SpreadLayout,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::as_inner_mut(self)
    }
}
