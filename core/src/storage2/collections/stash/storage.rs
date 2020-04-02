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
use crate::{
    storage2 as storage,
    storage2::{
        ClearForward,
        KeyPtr,
        Pack,
        PullAt,
        PullForward,
        PushAt,
        PushForward,
        StorageFootprint,
    },
};
use ink_primitives::Key;

impl<T> StorageFootprint for StorageStash<T>
where
    T: StorageFootprint,
{
    const VALUE: u64 = 1 + <storage::LazyIndexMap<T> as StorageFootprint>::VALUE;
}

impl<T> PullForward for StorageStash<T>
where
    T: scale::Decode,
    storage::LazyIndexMap<Pack<Entry<T>>>: PullForward,
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

impl<T> PushForward for StorageStash<T>
where
    storage::LazyIndexMap<Pack<Entry<T>>>: PushForward,
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
