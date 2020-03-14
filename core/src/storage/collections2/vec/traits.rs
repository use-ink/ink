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

use super::Vec as StorageVec;
use crate::{
    storage,
    storage::{
        ClearForward,
        KeyPtr,
        PullForward,
        PushForward,
        SaturatingStorage,
        StorageFootprint,
        StorageFootprintOf,
    },
};
use core::{
    iter::FromIterator,
    ops::Add,
};
use typenum::{
    Add1,
    Integer,
};

impl<T> core::ops::Index<u32> for StorageVec<T>
where
    T: StorageFootprint + PullForward,
{
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T> core::ops::IndexMut<u32> for StorageVec<T>
where
    T: SaturatingStorage + StorageFootprint + PullForward,
{
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

impl<'a, T: 'a> IntoIterator for &'a StorageVec<T>
where
    T: StorageFootprint + PullForward,
{
    type Item = &'a T;
    type IntoIter = super::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> FromIterator<T> for StorageVec<T>
where
    T: StorageFootprint + SaturatingStorage,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut vec = StorageVec::new();
        for item in iter {
            vec.push(item)
        }
        vec
    }
}

impl<T> StorageFootprint for StorageVec<T>
where
    T: StorageFootprint,
    storage::LazyChunk<T>: StorageFootprint,
    StorageFootprintOf<storage::LazyChunk<T>>: Add<typenum::B1>,
    Add1<StorageFootprintOf<storage::LazyChunk<T>>>: Integer,
{
    type Value = Add1<StorageFootprintOf<storage::LazyChunk<T>>>;
}

impl<T> PullForward for StorageVec<T>
where
    T: StorageFootprint,
    storage::LazyChunk<T>: PullForward,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self {
            len: PullForward::pull_forward(ptr),
            elems: PullForward::pull_forward(ptr),
        }
    }
}

impl<T> PushForward for StorageVec<T>
where
    storage::LazyChunk<T>: PushForward,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        PushForward::push_forward(&self.len(), ptr);
        PushForward::push_forward(&self.elems, ptr);
    }
}

impl<T> ClearForward for StorageVec<T>
where
    T: StorageFootprint + ClearForward + PullForward,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        ClearForward::clear_forward(&self.len(), ptr);
        if self.elems.key().is_none() {
            return
        }
        for (index, elem) in self.iter().enumerate() {
            <T as ClearForward>::clear_forward(
                elem,
                &mut KeyPtr::from(
                    self.elems
                        .key_at(&(index as u32))
                        .expect("expected a key mapping since self.elems.key() is some"),
                ),
            )
        }
    }
}
