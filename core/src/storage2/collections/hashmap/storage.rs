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
    HashMap as StorageHashMap,
    KeyIndex,
    ValueEntry,
};
use crate::{
    hash::hasher::Hasher,
    storage2::{
        collections::Stash as StorageStash,
        traits2::{
            forward_clear_packed,
            forward_pull_packed,
            forward_push_packed,
            KeyPtr as KeyPtr2,
            PackedLayout,
            SpreadLayout,
        },
        ClearAt,
        ClearForward,
        KeyPtr,
        PullAt,
        PullForward,
        PushAt,
        PushForward,
        StorageFootprint,
    },
};
use ink_primitives::Key;

impl<T> SpreadLayout for ValueEntry<T>
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

impl<T> PackedLayout for ValueEntry<T>
where
    T: PackedLayout,
{
    fn pull_packed(&mut self, at: &Key) {
        <T as PackedLayout>::pull_packed(&mut self.value, at)
    }

    fn push_packed(&self, at: &Key) {
        <T as PackedLayout>::push_packed(&self.value, at)
    }

    fn clear_packed(&self, at: &Key) {
        <T as PackedLayout>::clear_packed(&self.value, at)
    }
}

impl<K, V, H, O> SpreadLayout for StorageHashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    K: StorageFootprint + PullForward,
    V: PackedLayout,
    H: Hasher<Output = O>,
    O: Default,
    Key: From<O>,
{
    const FOOTPRINT: u64 = 1 + <StorageStash<K> as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr2) -> Self {
        Self {
            keys: SpreadLayout::pull_spread(ptr),
            values: SpreadLayout::pull_spread(ptr),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr2) {
        SpreadLayout::push_spread(&self.keys, ptr);
        SpreadLayout::push_spread(&self.values, ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr2) {
        for key in self.keys() {
            // It might seem wasteful to clear all entries instead of just
            // the occupied ones. However this spares us from having one extra
            // read for every element in the storage stash to filter out vacant
            // entries. So this is actually a trade-off and at the time of this
            // implementation it is unclear which path is more efficient.
            //
            // The bet is that clearing a storage cell is cheaper than reading one.
            self.values.clear_packed_at(key);
        }
        SpreadLayout::clear_spread(&self.keys, ptr);
        SpreadLayout::clear_spread(&self.values, ptr);
    }
}

impl<K, V, H> StorageFootprint for StorageHashMap<K, V, H>
where
    K: Ord + StorageFootprint,
    H: Hasher,
{
    const VALUE: u64 = 1 + <StorageStash<K> as StorageFootprint>::VALUE;
}

impl<K, V, H> PullForward for StorageHashMap<K, V, H>
where
    K: Ord + scale::Decode + PullForward + StorageFootprint,
    V: StorageFootprint,
    H: Hasher,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self {
            keys: PullForward::pull_forward(ptr),
            values: PullForward::pull_forward(ptr),
        }
    }
}

impl<V> StorageFootprint for ValueEntry<V>
where
    V: StorageFootprint,
{
    const VALUE: u64 = <V as StorageFootprint>::VALUE + <u32 as StorageFootprint>::VALUE;
}

impl<V> PushForward for ValueEntry<V>
where
    V: StorageFootprint + PushForward,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        <V as PushForward>::push_forward(&self.value, ptr);
        <KeyIndex as PushForward>::push_forward(&self.key_index, ptr);
    }
}

impl<V> PushAt for ValueEntry<V>
where
    V: scale::Encode,
{
    fn push_at(&self, at: Key) {
        crate::env::set_contract_storage::<Self>(at, self)
    }
}

impl<V> PullForward for ValueEntry<V>
where
    V: StorageFootprint + PullForward,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self {
            value: <V as PullForward>::pull_forward(ptr),
            key_index: <KeyIndex as PullForward>::pull_forward(ptr),
        }
    }
}

impl<V> PullAt for ValueEntry<V>
where
    V: scale::Decode,
{
    fn pull_at(at: Key) -> Self {
        crate::storage2::pull_single_cell(at)
    }
}

impl<V> ClearForward for ValueEntry<V>
where
    V: ClearForward,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        <V as ClearForward>::clear_forward(&self.value, ptr);
        <KeyIndex as ClearForward>::clear_forward(&self.key_index, ptr);
    }
}

impl<V> ClearAt for ValueEntry<V> {
    fn clear_at(&self, at: Key) {
        crate::env::clear_contract_storage(at)
    }
}

impl<K, V, H> PushForward for StorageHashMap<K, V, H>
where
    K: PushAt + scale::Codec + Ord,
    V: PushAt + scale::Encode,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        PushForward::push_forward(&self.keys, ptr);
        PushForward::push_forward(&self.values, ptr);
    }
}

impl<K, V, H> ClearForward for StorageHashMap<K, V, H>
where
    K: StorageFootprint + ClearForward + PullForward + scale::Codec + Ord + Clone,
    V: ClearForward + scale::Encode,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        ClearForward::clear_forward(&self.keys, ptr);
        ClearForward::clear_forward(&self.values, ptr);
    }
}
