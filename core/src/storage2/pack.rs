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
    KeyPtr,
    PullAt,
    PullForward,
    PushAt,
    PushForward,
    SaturatingStorage,
    StorageFootprint,
};
use ink_primitives::Key;

/// Packs the inner `T` so that it only occupies a single contract stoage cell.
///
/// # Note
///
/// This is an important modular building stone in order to manage contract
/// storage occupation. By default types try to distribute themselves onto
/// their respective contract storage area. However, upon packing them into
/// `Pack<T>` they will be compressed to only ever make use of a single
/// contract storage cell. Sometimes this can be advantageous for performance
/// reasons.
///
/// # Usage
///
/// - A `Pack<i32>` is equivalent to `i32` in its storage occupation.
/// - A `Pack<(i32, i32)>` will occupy a single cell compared to `(i32, i32)`
///   which occupies a cell per `i32`.
/// - A `Lazy<Pack<[u8; 8]>>` lazily loads a `Pack<[u8; 8]>` which occupies
///   a single cell whereas a `[u8; 8]` would occupy 8 cells in total - one for
///   each `u8`.
/// - Rust collections will never use more than a single cell. So
///   `Pack<LinkedList<T>>` and `LinkedList<T>` will occupy the same amount of
///   cells, namely 1.
/// - Packs can be packed. So for example a
///   `Pack<(Pack<(i32, i32)>, Pack<[u8; 8]>)` uses just one cell instead of
///   two cells which is the case for `(Pack<(i32, i32)>, Pack<[u8; 8]>)`.
/// - Not all `storage` types can be packed. Only those that are implementing
///   `PullAt` and `PushAt`. For example `storage::Vec<T>` does not implement
///   those trait and thus cannot be packed.
///
/// As a general advice pack values together that are frequently used together.
/// Also pack many very small elements (e.g. `u8`, `bool`, `u16`) together.
#[derive(Debug, Copy, Clone, scale::Encode, scale::Decode)]
pub struct Pack<T> {
    /// The packed `T` value.
    inner: T,
}

impl<T> Pack<T> {
    /// Creates a new packed value.
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }

    /// Returns the packed value.
    pub fn into_inner(pack: Self) -> T {
        pack.inner
    }

    /// Returns a shared reference to the packed value.
    pub fn as_inner(pack: &Pack<T>) -> &T {
        pack.get()
    }

    /// Returns an exclusive reference to the packed value.
    pub fn as_inner_mut(pack: &mut Pack<T>) -> &mut T {
        pack.get_mut()
    }

    /// Returns a shared reference to the packed value.
    fn get(&self) -> &T {
        &self.inner
    }

    /// Returns an exclusive reference to the packed value.
    fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> StorageFootprint for Pack<T> {
    type Value = typenum::U1;
    const VALUE: u64 = 1;
}

impl<T> SaturatingStorage for Pack<T> {}

impl<T> PullForward for Pack<T>
where
    T: PullAt,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        <Self as PullAt>::pull_at(ptr.next_for::<Self>())
    }
}

impl<T> PullAt for Pack<T>
where
    T: PullAt,
{
    fn pull_at(at: Key) -> Self {
        Self {
            inner: <T as PullAt>::pull_at(at),
        }
    }
}

impl<T> PushForward for Pack<T>
where
    T: PushAt,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        <Self as PushAt>::push_at(self, ptr.next_for::<Self>())
    }
}

impl<T> PushAt for Pack<T>
where
    T: PushAt,
{
    fn push_at(&self, at: Key) {
        <T as PushAt>::push_at(self.get(), at)
    }
}

impl<T> From<T> for Pack<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Default for Pack<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> core::ops::Deref for Pack<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> core::ops::DerefMut for Pack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<T> core::cmp::PartialEq for Pack<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.get(), other.get())
    }
}

impl<T> core::cmp::Eq for Pack<T> where T: Eq {}

impl<T> core::cmp::PartialOrd for Pack<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(self.get(), other.get())
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(self.get(), other.get())
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(self.get(), other.get())
    }
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(self.get(), other.get())
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(self.get(), other.get())
    }
}

impl<T> core::cmp::Ord for Pack<T>
where
    T: core::cmp::Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(self.get(), other.get())
    }
}

impl<T> core::fmt::Display for Pack<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(self.get(), f)
    }
}

impl<T> core::hash::Hash for Pack<T>
where
    T: core::hash::Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<T> core::convert::AsRef<T> for Pack<T> {
    fn as_ref(&self) -> &T {
        self.get()
    }
}

impl<T> core::convert::AsMut<T> for Pack<T> {
    fn as_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T> ink_prelude::borrow::Borrow<T> for Pack<T> {
    fn borrow(&self) -> &T {
        self.get()
    }
}

impl<T> ink_prelude::borrow::BorrowMut<T> for Pack<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}
