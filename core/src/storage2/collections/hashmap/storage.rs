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
