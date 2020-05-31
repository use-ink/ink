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
    traits::{
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    },
};

#[cfg(feature = "std")]
const _: () = {
    use crate::storage2::{
        lazy::Lazy,
        traits::StorageLayout,
    };
    use ink_abi::layout2::{
        FieldLayout,
        Layout,
        StructLayout,
    };
    use type_metadata::Metadata;

    impl<T> StorageLayout for StorageVec<T>
    where
        T: PackedLayout + Metadata,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Struct(StructLayout::new(vec![
                FieldLayout::new("len", <Lazy<u32> as StorageLayout>::layout(key_ptr)),
                FieldLayout::new(
                    "elems",
                    <LazyIndexMap<T> as StorageLayout>::layout(key_ptr),
                ),
            ]))
        }
    }
};

impl<T> SpreadLayout for StorageVec<T>
where
    T: PackedLayout,
{
    const FOOTPRINT: u64 = 1 + <LazyIndexMap<T> as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self {
            len: SpreadLayout::pull_spread(ptr),
            elems: SpreadLayout::pull_spread(ptr),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::push_spread(&self.len, ptr);
        SpreadLayout::push_spread(&self.elems, ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        self.clear_cells();
        SpreadLayout::clear_spread(&self.len, ptr);
        SpreadLayout::clear_spread(&self.elems, ptr);
    }
}
