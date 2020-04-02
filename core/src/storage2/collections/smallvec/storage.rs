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

use super::SmallVec;
use crate::storage2::{
    KeyPtr,
    LazyArray,
    LazyArrayLength,
    PullForward,
    PushForward,
    StorageFootprint,
    StorageFootprintOf,
};
use core::ops::Add;
use typenum::{
    Add1,
    Unsigned,
};

impl<T, N> StorageFootprint for SmallVec<T, N>
where
    T: StorageFootprint + PullForward,
    N: LazyArrayLength<T>,
    LazyArray<T, N>: StorageFootprint,
    StorageFootprintOf<LazyArray<T, N>>: Add<typenum::B1>,
    Add1<StorageFootprintOf<LazyArray<T, N>>>: Unsigned,
{
    type Value = Add1<StorageFootprintOf<LazyArray<T, N>>>;
    const VALUE: u64 = 1 + <N as Unsigned>::U64;
}

impl<T, N> PullForward for SmallVec<T, N>
where
    N: LazyArrayLength<T>,
    LazyArray<T, N>: PullForward,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self {
            len: PullForward::pull_forward(ptr),
            elems: PullForward::pull_forward(ptr),
        }
    }
}

impl<T, N> PushForward for SmallVec<T, N>
where
    LazyArray<T, N>: PushForward,
    N: LazyArrayLength<T>,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        PushForward::push_forward(&self.len(), ptr);
        PushForward::push_forward(&self.elems, ptr);
    }
}
