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

//! Implementation of ink! storage traits.

use super::{
    HashMap as StorageHashMap,
    ValueEntry,
};
use crate::{
    collections::Stash as StorageStash,
    traits::{
        forward_clear_packed,
        forward_pull_packed,
        forward_push_packed,
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    },
};
use ink_env::hash::{
    CryptoHash,
    HashOutput,
};
use ink_primitives::Key;

#[cfg(feature = "std")]
const _: () = {
    use crate::{
        lazy::LazyHashMap,
        traits::{
            LayoutCryptoHasher,
            StorageLayout,
        },
    };
    use ink_metadata::layout::{
        FieldLayout,
        Layout,
        StructLayout,
    };
    use scale_info::TypeInfo;

    impl<K, V, H> StorageLayout for StorageHashMap<K, V, H>
    where
        K: TypeInfo + Ord + Clone + PackedLayout + 'static,
        V: TypeInfo + PackedLayout + 'static,
        H: LayoutCryptoHasher + CryptoHash,
        Key: From<<H as HashOutput>::Type>,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Struct(StructLayout::new(vec![
                FieldLayout::new(
                    "keys",
                    <StorageStash<K> as StorageLayout>::layout(key_ptr),
                ),
                FieldLayout::new(
                    "values",
                    <LazyHashMap<K, ValueEntry<V>, H> as StorageLayout>::layout(key_ptr),
                ),
            ]))
        }
    }
};

impl<T> SpreadLayout for ValueEntry<T>
where
    T: PackedLayout,
{
    const FOOTPRINT: u64 = 1;
    const REQUIRES_DEEP_CLEAN_UP: bool = <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP;

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

impl<T> PackedLayout for ValueEntry<T>
where
    T: PackedLayout,
{
    fn pull_packed(&mut self, at: &Key) {
        <T as PackedLayout>::pull_packed(&mut self.value, at)
    }

    fn push_packed(&self, at: &Key) {
        <T as PackedLayout>::push_packed(&self.value, at)
    }

    fn clear_packed(&self, at: &Key) {
        <T as PackedLayout>::clear_packed(&self.value, at)
    }
}

impl<K, V, H> SpreadLayout for StorageHashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    const FOOTPRINT: u64 = 1 + <StorageStash<K> as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self {
            keys: SpreadLayout::pull_spread(ptr),
            values: SpreadLayout::pull_spread(ptr),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::push_spread(&self.keys, ptr);
        SpreadLayout::push_spread(&self.values, ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        self.clear_cells();
        SpreadLayout::clear_spread(&self.keys, ptr);
        SpreadLayout::clear_spread(&self.values, ptr);
    }
}
