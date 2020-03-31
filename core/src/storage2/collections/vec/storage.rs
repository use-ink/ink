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

use super::Vec as StorageVec;
use crate::{
    storage2 as storage,
    storage2::{
        ClearForward,
        KeyPtr,
        PullForward,
        PushForward,
        StorageFootprint,
        StorageFootprintOf,
    },
};
use core::ops::Add;
use typenum::{
    Add1,
    Unsigned,
};

impl<T> StorageFootprint for StorageVec<T>
where
    T: StorageFootprint,
    storage::LazyIndexMap<T>: StorageFootprint,
    StorageFootprintOf<storage::LazyIndexMap<T>>: Add<typenum::B1>,
    Add1<StorageFootprintOf<storage::LazyIndexMap<T>>>: Unsigned,
{
    type Value = Add1<StorageFootprintOf<storage::LazyIndexMap<T>>>;
}

impl<T> PullForward for StorageVec<T>
where
    T: StorageFootprint,
    storage::LazyIndexMap<T>: PullForward,
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
    storage::LazyIndexMap<T>: PushForward,
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
                        .key_at(index as u32)
                        .expect("expected a key mapping since self.elems.key() is some"),
                ),
            )
        }
    }
}
