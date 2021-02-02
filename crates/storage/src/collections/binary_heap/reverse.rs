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

//! [`BinaryHeap`](`super::BinaryHeap`) is a max-heap by default, where the *largest* element will
//! be returned by `heap.pop()`. To use a [`BinaryHeap`](`super::BinaryHeap`) as a min-heap, where
//! the *smallest* element returned by `heap.pop()`, the type `T` of the binary heap can be wrapped
//! in a `Reverse<T>`.
//!
//! [`Reverse`] simply wraps [`core::cmp::Reverse`] and implements all the required traits for use
//! as a storage struct.

use crate::traits::{
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use ink_prelude::vec::Vec;
use ink_primitives::Key;

/// Wrapper for [`core::cmp::Reverse`] for using a [`BinaryHeap`](`super::BinaryHeap`) as a
/// min-heap.
#[derive(PartialEq, Eq, Ord, PartialOrd, Debug, Copy, Clone, Default)]
pub struct Reverse<T>(core::cmp::Reverse<T>);

impl<T> Reverse<T>
where
    T: PackedLayout + Ord,
{
    /// Construct a new [`Reverse`] from the given value.
    pub fn new(value: T) -> Self {
        Self(core::cmp::Reverse(value))
    }

    /// Return a shared reference to the inner value.
    pub fn value(&self) -> &T {
        &(self.0).0
    }
}

impl<T> SpreadLayout for Reverse<T>
where
    T: PackedLayout + Ord,
{
    const FOOTPRINT: u64 = <T as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self::new(SpreadLayout::pull_spread(ptr))
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::push_spread(self.value(), ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::clear_spread(self.value(), ptr);
    }
}

impl<T> PackedLayout for Reverse<T>
where
    T: PackedLayout + Ord,
{
    fn pull_packed(&mut self, at: &Key) {
        <T as PackedLayout>::pull_packed(&mut (self.0).0, at)
    }

    fn push_packed(&self, at: &Key) {
        <T as PackedLayout>::push_packed(&(self.0).0, at)
    }

    fn clear_packed(&self, at: &Key) {
        <T as PackedLayout>::clear_packed(&(self.0).0, at)
    }
}

impl<T> scale::Encode for Reverse<T>
where
    T: PackedLayout + Ord + scale::Encode,
{
    #[inline]
    fn size_hint(&self) -> usize {
        <T as scale::Encode>::size_hint(self.value())
    }

    #[inline]
    fn encode_to<O: scale::Output + ?Sized>(&self, dest: &mut O) {
        <T as scale::Encode>::encode_to(self.value(), dest)
    }

    #[inline]
    fn encode(&self) -> Vec<u8> {
        <T as scale::Encode>::encode(self.value())
    }

    #[inline]
    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        <T as scale::Encode>::using_encoded(self.value(), f)
    }
}

impl<T> scale::Decode for Reverse<T>
where
    T: PackedLayout + Ord + scale::Decode,
{
    fn decode<I: scale::Input>(value: &mut I) -> Result<Self, scale::Error> {
        let value = <T as scale::Decode>::decode(value)?;
        Ok(Self::new(value))
    }
}
