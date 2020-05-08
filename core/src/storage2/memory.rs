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
    KeyPtr,
    SpreadLayout,
};
use core::{
    convert::{
        self,
        AsRef,
    },
    fmt,
    fmt::Display,
    ops::{
        Deref,
        DerefMut,
    },
};
use ink_prelude::borrow::{
    Borrow,
    BorrowMut,
};

/// An instance that is solely stored within the contract's memory.
///
/// This will never be stored to or loaded from contract storage.
///
/// # Note
///
/// Use instances of this type in order to have some shared state between
/// contract messages and functions.
/// Its usage is comparable to the Solidity's `memory` instances.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Memory<T> {
    /// The inner value that will always be stored within contract memory.
    inner: T,
}

impl<T> SpreadLayout for Memory<T>
where
    T: Default,
{
    const FOOTPRINT: u64 = 0;

    fn pull_spread(_ptr: &mut KeyPtr) -> Self {
        Default::default()
    }

    fn push_spread(&self, _ptr: &mut KeyPtr) {}
    fn clear_spread(&self, _ptr: &mut KeyPtr) {}
}

impl<T> Memory<T> {
    /// Returns a shared reference to the inner `T`.
    pub fn get(self: &Self) -> &T {
        &self.inner
    }

    /// Returns an exclusive reference to the inner `T`.
    pub fn get_mut(self: &mut Self) -> &mut T {
        &mut self.inner
    }
}

impl<T> From<T> for Memory<T> {
    fn from(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Display for Memory<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        core::fmt::Display::fmt(Self::get(self), f)
    }
}

impl<T> Default for Memory<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::from(<T as Default>::default())
    }
}

impl<T> Deref for Memory<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::get(self)
    }
}

impl<T> DerefMut for Memory<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::get_mut(self)
    }
}

impl<T> AsRef<T> for Memory<T>
where
    T: SpreadLayout,
{
    fn as_ref(&self) -> &T {
        Self::get(self)
    }
}

impl<T> convert::AsMut<T> for Memory<T>
where
    T: SpreadLayout,
{
    fn as_mut(&mut self) -> &mut T {
        Self::get_mut(self)
    }
}

impl<T> Borrow<T> for Memory<T>
where
    T: SpreadLayout,
{
    fn borrow(&self) -> &T {
        Self::get(self)
    }
}

impl<T> BorrowMut<T> for Memory<T>
where
    T: SpreadLayout,
{
    fn borrow_mut(&mut self) -> &mut T {
        Self::get_mut(self)
    }
}
