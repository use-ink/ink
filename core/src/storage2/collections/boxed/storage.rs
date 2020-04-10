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

use super::Box as StorageBox;
use crate::storage2::{
    alloc::DynamicAllocation,
    lazy::Lazy,
    ClearAt,
    ClearForward,
    KeyPtr,
    PullAt,
    PullForward,
    PushAt,
    PushForward,
    StorageFootprint,
};
use ink_primitives::Key;

impl<T> StorageFootprint for StorageBox<T>
where
    T: ClearForward + StorageFootprint,
{
    /// A boxed entity always uses exactly 1 cell for its storage.
    ///
    /// The indirectly stored storage entity is not considered because the
    /// `StorageSize` is only concerned with inplace storage usage.
    const VALUE: u64 = 1;
}

impl<T> PullForward for StorageBox<T>
where
    T: ClearForward + StorageFootprint,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        <Self as PullAt>::pull_at(ptr.next_for::<Self>())
    }
}

impl<T> PullAt for StorageBox<T>
where
    T: ClearForward + StorageFootprint,
{
    fn pull_at(at: Key) -> Self {
        let allocation = <DynamicAllocation as PullAt>::pull_at(at);
        Self {
            allocation,
            value: Lazy::lazy(allocation.key()),
        }
    }
}

impl<T> PushForward for StorageBox<T>
where
    T: ClearForward + PushForward + StorageFootprint,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        PushForward::push_forward(&self.allocation, ptr);
        PushForward::push_forward(&self.value, &mut KeyPtr::from(self.key()));
    }
}

impl<T> PushAt for StorageBox<T>
where
    T: ClearForward + PushForward + StorageFootprint,
{
    fn push_at(&self, at: Key) {
        <DynamicAllocation as PushAt>::push_at(&self.allocation, at);
        PushForward::push_forward(&self.value, &mut KeyPtr::from(self.key()));
    }
}

impl<T> ClearForward for StorageBox<T>
where
    T: ClearForward + StorageFootprint,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        <Self as ClearAt>::clear_at(self, ptr.next_for::<Self>())
    }
}

impl<T> ClearAt for StorageBox<T>
where
    T: ClearForward + StorageFootprint,
{
    fn clear_at(&self, at: Key) {
        <DynamicAllocation as ClearAt>::clear_at(&self.allocation, at);
        ClearForward::clear_forward(&self.value, &mut KeyPtr::from(self.key()));
    }
}
