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
use crate::storage2::{
    lazy::LazyIndexMap,
    ClearForward,
    KeyPtr,
    PullForward,
    PushForward,
    StorageFootprint,
};

impl<T> StorageFootprint for StorageVec<T>
where
    T: StorageFootprint,
{
    const VALUE: u64 = 1 + <LazyIndexMap<T> as StorageFootprint>::VALUE;
}

impl<T> PullForward for StorageVec<T>
where
    T: StorageFootprint,
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
    T: PushForward + PullForward + StorageFootprint,
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
        ClearForward::clear_forward(&self.len, ptr);
        ClearForward::clear_forward(&self.elems, ptr);
    }
}
