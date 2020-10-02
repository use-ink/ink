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

use super::BinaryHeap;
use crate::storage2::{
    collections::{
        binary_heap::{
            wrapper::Wrapper,
            Group,
        },
        Vec as StorageVec,
    },
    traits::{
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    },
};

#[cfg(feature = "std")]
const _: () = {
    use crate::storage2::traits::StorageLayout;
    use ink_metadata::layout2::{
        FieldLayout,
        Layout,
        StructLayout,
    };
    use scale_info::TypeInfo;

    impl<T> StorageLayout for BinaryHeap<T>
    where
        T: PackedLayout + Ord + TypeInfo + 'static,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Struct(StructLayout::new(vec![FieldLayout::new(
                "elems",
                <Wrapper<T> as StorageLayout>::layout(key_ptr),
            )]))
        }
    }
};

#[cfg(feature = "std")]
const _: () = {
    use crate::storage2::{
        lazy::Lazy,
        traits::StorageLayout,
    };
    use ink_metadata::layout2::{
        FieldLayout,
        Layout,
        StructLayout,
    };
    use scale_info::TypeInfo;

    impl<T> StorageLayout for Wrapper<T>
    where
        T: PackedLayout + Ord + TypeInfo + 'static,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Struct(StructLayout::new(vec![
                FieldLayout::new("len", <Lazy<u32> as StorageLayout>::layout(key_ptr)),
                FieldLayout::new(
                    "elems",
                    <StorageVec<Group<T>> as StorageLayout>::layout(key_ptr),
                ),
            ]))
        }
    }
};

impl<T> SpreadLayout for BinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    const FOOTPRINT: u64 = 1 + <Wrapper<T> as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self {
            groups: SpreadLayout::pull_spread(ptr),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::push_spread(&self.groups, ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::clear_spread(&self.groups, ptr);
    }
}
