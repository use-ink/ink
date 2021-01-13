// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

use super::DynamicAllocation;
use crate::{
    collections::BitStash,
    traits::{
        KeyPtr,
        SpreadLayout,
    },
};

/// The dynamic allocator.
///
/// Manages dynamic storage allocations in a very efficient and economic way.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DynamicAllocator {
    allocations: BitStash,
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::{
        FieldLayout,
        Layout,
        StructLayout,
    };

    impl StorageLayout for DynamicAllocator {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Struct(StructLayout::new(vec![FieldLayout::new(
                "allocations",
                <BitStash as StorageLayout>::layout(key_ptr),
            )]))
        }
    }
};

impl SpreadLayout for DynamicAllocator {
    const FOOTPRINT: u64 = <BitStash as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self {
            allocations: SpreadLayout::pull_spread(ptr),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::push_spread(&self.allocations, ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::clear_spread(&self.allocations, ptr);
    }
}

impl DynamicAllocator {
    /// Returns a new dynamic storage allocation.
    ///
    /// # Panics
    ///
    /// If the dynamic allocator ran out of free dynamic allocations.
    pub fn alloc(&mut self) -> DynamicAllocation {
        DynamicAllocation(self.allocations.put())
    }

    /// Frees the given dynamic storage allocation.
    ///
    /// This makes the given dynamic storage allocation available again
    /// for new dynamic storage allocations.
    ///
    /// # Panics
    ///
    /// Panics if the given dynamic allocation is invalid.
    /// A dynamic allocation is invalid if it is not represented as occupied
    /// in the `free` list.
    pub fn free(&mut self, allocation: DynamicAllocation) {
        let index = allocation.get();
        if !self
            .allocations
            .take(index)
            .expect("invalid dynamic storage allocation")
        {
            panic!(
                "encountered double free of dynamic storage: at index {}",
                index
            )
        }
    }
}
