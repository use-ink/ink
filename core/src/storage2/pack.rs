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
    forward_clear_packed,
    forward_pull_packed,
    forward_push_packed,
    KeyPtr,
    PackedLayout,
    SpreadLayout,
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
        &pack.inner
    }

    /// Returns an exclusive reference to the packed value.
    pub fn as_inner_mut(pack: &mut Pack<T>) -> &mut T {
        &mut pack.inner
    }
}

impl<T> SpreadLayout for Pack<T>
where
    T: PackedLayout,
{
    const FOOTPRINT: u64 = 1;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Pack::from(forward_pull_packed::<T>(ptr))
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        forward_push_packed::<T>(Self::as_inner(self), ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        forward_clear_packed::<T>(Self::as_inner(self), ptr)
    }
}

impl<T> PackedLayout for Pack<T>
where
    T: PackedLayout,
{
    fn pull_packed(&mut self, at: &Key) {
        <T as PackedLayout>::pull_packed(Self::as_inner_mut(self), at)
    }
    fn push_packed(&self, at: &Key) {
        <T as PackedLayout>::push_packed(Self::as_inner(self), at)
    }
    fn clear_packed(&self, at: &Key) {
        <T as PackedLayout>::clear_packed(Self::as_inner(self), at)
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
        Self::as_inner(self)
    }
}

impl<T> core::ops::DerefMut for Pack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::as_inner_mut(self)
    }
}

impl<T> core::cmp::PartialEq for Pack<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(Self::as_inner(self), Self::as_inner(other))
    }
}

impl<T> core::cmp::Eq for Pack<T> where T: Eq {}

impl<T> core::cmp::PartialOrd for Pack<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(Self::as_inner(self), Self::as_inner(other))
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(Self::as_inner(self), Self::as_inner(other))
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(Self::as_inner(self), Self::as_inner(other))
    }
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(Self::as_inner(self), Self::as_inner(other))
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(Self::as_inner(self), Self::as_inner(other))
    }
}

impl<T> core::cmp::Ord for Pack<T>
where
    T: core::cmp::Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(Self::as_inner(self), Self::as_inner(other))
    }
}

impl<T> core::fmt::Display for Pack<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(Self::as_inner(self), f)
    }
}

impl<T> core::hash::Hash for Pack<T>
where
    T: core::hash::Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        Self::as_inner(self).hash(state);
    }
}

impl<T> core::convert::AsRef<T> for Pack<T> {
    fn as_ref(&self) -> &T {
        Self::as_inner(self)
    }
}

impl<T> core::convert::AsMut<T> for Pack<T> {
    fn as_mut(&mut self) -> &mut T {
        Self::as_inner_mut(self)
    }
}

impl<T> ink_prelude::borrow::Borrow<T> for Pack<T> {
    fn borrow(&self) -> &T {
        Self::as_inner(self)
    }
}

impl<T> ink_prelude::borrow::BorrowMut<T> for Pack<T> {
    fn borrow_mut(&mut self) -> &mut T {
        Self::as_inner_mut(self)
    }
}
