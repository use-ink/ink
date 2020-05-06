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
};
use ink_prelude::vec::Vec;
use ink_primitives::Key;

impl<T> SpreadLayout for StorageBox<T>
where
    T: SpreadLayout,
    T: StorageFootprint + ClearForward + PullForward,
{
    const FOOTPRINT: u64 = 1;

    fn pull_spread(ptr: &mut KeyPtr2) -> Self {
        forward_pull_packed::<Self>(ptr)
    }

    fn push_spread(&self, ptr: &mut KeyPtr2) {
        forward_push_packed::<Self>(&self, ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr2) {
        forward_clear_packed::<Self>(&self, ptr)
    }
}

impl<T> scale::Encode for StorageBox<T>
where
    T: SpreadLayout,
    T: StorageFootprint + ClearForward,
{
    fn size_hint(&self) -> usize {
        <DynamicAllocation as scale::Encode>::size_hint(&self.allocation)
    }

    fn encode_to<O: scale::Output>(&self, dest: &mut O) {
        <DynamicAllocation as scale::Encode>::encode_to(&self.allocation, dest)
    }

    fn encode(&self) -> Vec<u8> {
        <DynamicAllocation as scale::Encode>::encode(&self.allocation)
    }

    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        <DynamicAllocation as scale::Encode>::using_encoded(&self.allocation, f)
    }
}

impl<T> scale::Decode for StorageBox<T>
where
    T: SpreadLayout,
    T: StorageFootprint + ClearForward,
{
    fn decode<I: scale::Input>(value: &mut I) -> Result<Self, scale::Error> {
        Ok(StorageBox::lazy(
            <DynamicAllocation as scale::Decode>::decode(value)?,
        ))
    }
}

impl<T> PackedLayout for StorageBox<T>
where
    T: SpreadLayout,
    T: StorageFootprint + ClearForward + PullForward,
{
    fn pull_packed(&mut self, _at: &Key) {}

    fn push_packed(&self, _at: &Key) {
        <T as SpreadLayout>::push_spread(Self::get(self), &mut KeyPtr2::from(self.key()))
    }

    fn clear_packed(&self, _at: &Key) {
        <T as SpreadLayout>::clear_spread(Self::get(self), &mut KeyPtr2::from(self.key()))
    }
}

impl<T> StorageFootprint for StorageBox<T>
where
    T: SpreadLayout,
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
    T: SpreadLayout,
    T: ClearForward + StorageFootprint,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        <Self as PullAt>::pull_at(ptr.next_for::<Self>())
    }
}

impl<T> PullAt for StorageBox<T>
where
    T: SpreadLayout,
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
    T: SpreadLayout,
    T: ClearForward + PushForward + StorageFootprint,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        PushForward::push_forward(&self.allocation, ptr);
        PushForward::push_forward(&self.value, &mut KeyPtr::from(self.key()));
    }
}

impl<T> PushAt for StorageBox<T>
where
    T: SpreadLayout,
    T: ClearForward + PushForward + StorageFootprint,
{
    fn push_at(&self, at: Key) {
        <DynamicAllocation as PushAt>::push_at(&self.allocation, at);
        PushForward::push_forward(&self.value, &mut KeyPtr::from(self.key()));
    }
}

impl<T> ClearForward for StorageBox<T>
where
    T: SpreadLayout,
    T: ClearForward + StorageFootprint,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        <Self as ClearAt>::clear_at(self, ptr.next_for::<Self>())
    }
}

impl<T> ClearAt for StorageBox<T>
where
    T: SpreadLayout,
    T: ClearForward + StorageFootprint,
{
    fn clear_at(&self, at: Key) {
        <DynamicAllocation as ClearAt>::clear_at(&self.allocation, at);
        ClearForward::clear_forward(&self.value, &mut KeyPtr::from(self.key()));
    }
}
