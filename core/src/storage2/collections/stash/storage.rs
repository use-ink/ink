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

//! Implementation of ink! storage traits.

use super::{
    Entry,
    Header,
    Stash as StorageStash,
};
use crate::storage2::{
    lazy::LazyIndexMap,
    traits2::{
        forward_clear_packed,
        forward_pull_packed,
        forward_push_packed,
        KeyPtr as KeyPtr2,
        PackedLayout,
        SpreadLayout,
    },
    ClearForward,
    KeyPtr,
    PullAt,
    PullForward,
    PushAt,
    PushForward,
    StorageFootprint,
};
use ink_primitives::Key;

impl SpreadLayout for Header {
    const FOOTPRINT: u64 = 1;

    fn pull_spread(ptr: &mut KeyPtr2) -> Self {
        forward_pull_packed::<Self>(ptr)
    }

    fn push_spread(&self, ptr: &mut KeyPtr2) {
        forward_push_packed::<Self>(self, ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr2) {
        forward_clear_packed::<Self>(self, ptr)
    }
}

impl PackedLayout for Header {
    fn pull_packed(&mut self, _at: &Key) {}
    fn push_packed(&self, _at: &Key) {}
    fn clear_packed(&self, _at: &Key) {}
}

impl<T> SpreadLayout for Entry<T>
where
    T: PackedLayout,
{
    const FOOTPRINT: u64 = 1;

    fn pull_spread(ptr: &mut KeyPtr2) -> Self {
        forward_pull_packed::<Self>(ptr)
    }

    fn push_spread(&self, ptr: &mut KeyPtr2) {
        forward_push_packed::<Self>(self, ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr2) {
        forward_clear_packed::<Self>(self, ptr)
    }
}

impl<T> PackedLayout for Entry<T>
where
    T: PackedLayout,
{
    fn pull_packed(&mut self, at: &Key) {
        if let Entry::Occupied(value) = self {
            <T as PackedLayout>::pull_packed(value, at)
        }
    }

    fn push_packed(&self, at: &Key) {
        if let Entry::Occupied(value) = self {
            <T as PackedLayout>::push_packed(value, at)
        }
    }

    fn clear_packed(&self, at: &Key) {
        if let Entry::Occupied(value) = self {
            <T as PackedLayout>::clear_packed(value, at)
        }
    }
}

impl<T> SpreadLayout for StorageStash<T>
where
    T: PackedLayout,
    T: StorageFootprint + PullForward,
{
    const FOOTPRINT: u64 = 1 + <LazyIndexMap<T> as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr2) -> Self {
        Self {
            header: SpreadLayout::pull_spread(ptr),
            entries: SpreadLayout::pull_spread(ptr),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr2) {
        SpreadLayout::push_spread(&self.header, ptr);
        SpreadLayout::push_spread(&self.entries, ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr2) {
        for index in 0..self.len_entries() {
            // It might seem wasteful to clear all entries instead of just
            // the occupied ones. However this spares us from having one extra
            // read for every element in the storage stash to filter out vacant
            // entries. So this is actually a trade-off and at the time of this
            // implementation it is unclear which path is more efficient.
            //
            // The bet is that clearing a storage cell is cheaper than reading one.
            self.entries.clear_packed_at(index);
        }
        SpreadLayout::clear_spread(&self.header, ptr);
        SpreadLayout::clear_spread(&self.entries, ptr);
    }
}

impl<T> StorageFootprint for StorageStash<T>
where
    T: StorageFootprint,
{
    const VALUE: u64 = 1 + <LazyIndexMap<T> as StorageFootprint>::VALUE;
}

impl<T> PullForward for StorageStash<T>
where
    T: scale::Decode,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self {
            header: PullForward::pull_forward(ptr),
            entries: PullForward::pull_forward(ptr),
        }
    }
}

impl PullAt for Header {
    fn pull_at(at: Key) -> Self {
        crate::storage2::pull_single_cell(at)
    }
}

impl PushAt for Header {
    fn push_at(&self, at: Key) {
        crate::env::set_contract_storage::<Self>(at, self)
    }
}

impl<T> PullAt for Entry<T>
where
    T: scale::Decode,
{
    fn pull_at(at: Key) -> Self {
        crate::storage2::pull_single_cell(at)
    }
}

impl<T> PushAt for Entry<T>
where
    T: scale::Encode,
{
    fn push_at(&self, at: Key) {
        crate::env::set_contract_storage(at, self)
    }
}

impl<T> PushForward for StorageStash<T>
where
    T: PushAt + scale::Codec,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        PushForward::push_forward(&self.header, ptr);
        PushForward::push_forward(&self.entries, ptr);
    }
}

impl<T> ClearForward for StorageStash<T>
where
    T: StorageFootprint + ClearForward + PullForward + scale::Decode,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        ClearForward::clear_forward(&self.len(), ptr);
        if self.entries.key().is_none() {
            return
        }
        for (index, elem) in self.iter().enumerate() {
            <T as ClearForward>::clear_forward(
                elem,
                &mut KeyPtr::from(
                    self.entries
                        .key_at(index as u32)
                        .expect("expected a key mapping since self.elems.key() is some"),
                ),
            )
        }
    }
}
