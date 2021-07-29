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

mod impls;
mod iter;

use crate::{
    collections::slice::iter::{
        Iter,
        IterMut,
    },
    lazy::{
        LazyArray,
        LazyIndexMap,
    },
    traits::PackedLayout,
};
use core::ops::Range;

/// A view into a storage `Vec`.
#[derive(Clone, Debug)]
pub struct Slice<T> {
    /// The start and end indices inside the `index_map`. Indexing the slice using `n` means that we
    /// access `n + range.start`.
    range: Range<u32>,
    backing_storage: T,
}

impl<T> Slice<T>
where
    T: ContiguousStorage,
{
    pub fn new(range: Range<u32>, backing_storage: T) -> Slice<T> {
        Slice {
            range,
            backing_storage,
        }
    }

    pub fn get(&self, index: u32) -> Option<&T::Item> {
        self.backing_storage.get(index + self.range.start)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            index: 0,
            range: self.range.clone(),
            backing_storage: &self.backing_storage,
        }
    }
}

/// A view into a storage `Vec`.
#[derive(Clone, Debug)]
pub struct SliceMut<T> {
    /// The start and end indices inside the `index_map`. Indexing the slice using `n` means that we
    /// access `n + range.start`.
    range: Range<u32>,
    backing_storage: T,
}

impl<T> SliceMut<T>
where
    T: ContiguousStorage,
{
    pub unsafe fn new(range: Range<u32>, backing_storage: T) -> SliceMut<T> {
        SliceMut {
            range,
            backing_storage,
        }
    }

    pub fn get(&self, index: u32) -> Option<&T::Item> {
        self.backing_storage.get(index + self.range.start)
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut T::Item> {
        unsafe { self.backing_storage.get_mut(index) }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            index: 0,
            range: self.range.clone(),
            backing_storage: &self.backing_storage,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            index: 0,
            range: self.range.clone(),
            backing_storage: &self.backing_storage,
        }
    }
}

pub trait ContiguousStorage {
    type Item;

    unsafe fn get_mut(&self, index: u32) -> Option<&mut Self::Item>;
    fn get(&self, index: u32) -> Option<&Self::Item>;
}

impl<T> ContiguousStorage for &LazyIndexMap<T>
where
    T: PackedLayout,
{
    type Item = T;

    unsafe fn get_mut(&self, index: u32) -> Option<&mut T> {
        // SAFETY:
        //  - lazily_load requires that there is exclusive access to the T. The contract of
        //    ContiguousStorage ensures this variant.
        //  - lazily_load always returns a valid pointer.
        self.lazily_load(index).as_mut().value_mut().as_mut()
    }

    fn get(&self, index: u32) -> Option<&Self::Item> {
        LazyIndexMap::get(self, index)
    }
}

impl<T, const N: usize> ContiguousStorage for &LazyArray<T, N>
where
    T: PackedLayout,
{
    type Item = T;

    unsafe fn get_mut(&self, index: u32) -> Option<&mut T> {
        self.cached_entries
            .get_entry_mut(index)
            .map(|i| i.value_mut().as_mut())
            .flatten()
    }

    fn get(&self, index: u32) -> Option<&Self::Item> {
        LazyArray::get(self, index)
    }
}
