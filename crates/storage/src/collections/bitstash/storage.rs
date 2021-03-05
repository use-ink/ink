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

use super::{
    BitStash,
    CountFree,
};
use crate::{
    collections::{
        Bitvec as StorageBitvec,
        Vec as StorageVec,
    },
    traits::{
        forward_clear_packed,
        forward_pull_packed,
        forward_push_packed,
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    },
};
use ink_primitives::Key;

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::{
        FieldLayout,
        Layout,
        StructLayout,
    };

    impl StorageLayout for BitStash {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Struct(StructLayout::new(vec![
                FieldLayout::new(
                    "counts",
                    <StorageVec<CountFree> as StorageLayout>::layout(key_ptr),
                ),
                FieldLayout::new(
                    "elems",
                    <StorageBitvec as StorageLayout>::layout(key_ptr),
                ),
            ]))
        }
    }
};

impl SpreadLayout for CountFree {
    const FOOTPRINT: u64 = 1;
    const REQUIRES_DEEP_CLEAN_UP: bool = false;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        forward_pull_packed::<Self>(ptr)
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        forward_push_packed::<Self>(self, ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        forward_clear_packed::<Self>(self, ptr)
    }
}

impl PackedLayout for CountFree {
    fn pull_packed(&mut self, _at: &Key) {}
    fn push_packed(&self, _at: &Key) {}
    fn clear_packed(&self, _at: &Key) {}
}

impl SpreadLayout for BitStash {
    const FOOTPRINT: u64 = <StorageVec<CountFree> as SpreadLayout>::FOOTPRINT
        + <StorageBitvec as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self {
            counts: SpreadLayout::pull_spread(ptr),
            free: SpreadLayout::pull_spread(ptr),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::push_spread(&self.counts, ptr);
        SpreadLayout::push_spread(&self.free, ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::clear_spread(&self.counts, ptr);
        SpreadLayout::clear_spread(&self.free, ptr);
    }
}
