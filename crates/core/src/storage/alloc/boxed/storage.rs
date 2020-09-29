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
use crate::storage::{
    alloc::DynamicAllocation,
    traits::{
        forward_clear_packed,
        forward_pull_packed,
        forward_push_packed,
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    },
};
use ink_prelude::vec::Vec;
use ink_primitives::Key;

#[cfg(feature = "std")]
const _: () = {
    use crate::storage::traits::StorageLayout;
    use ink_metadata::layout::{
        CellLayout,
        Layout,
        LayoutKey,
    };

    impl<T> StorageLayout for StorageBox<T>
    where
        T: SpreadLayout,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Cell(CellLayout::new::<DynamicAllocation>(LayoutKey::from(
                key_ptr.advance_by(1),
            )))
        }
    }
};

impl<T> SpreadLayout for StorageBox<T>
where
    T: SpreadLayout,
{
    const FOOTPRINT: u64 = 1;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        forward_pull_packed::<Self>(ptr)
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        forward_push_packed::<Self>(&self, ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        forward_clear_packed::<Self>(&self, ptr)
    }
}

impl<T> scale::Encode for StorageBox<T>
where
    T: SpreadLayout,
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
{
    fn pull_packed(&mut self, _at: &Key) {}

    fn push_packed(&self, _at: &Key) {
        <T as SpreadLayout>::push_spread(Self::get(self), &mut KeyPtr::from(self.key()))
    }

    fn clear_packed(&self, _at: &Key) {
        <T as SpreadLayout>::clear_spread(Self::get(self), &mut KeyPtr::from(self.key()))
    }
}
