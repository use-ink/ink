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

use crate::traits::{
    impls::{
        forward_clear_packed,
        forward_pull_packed,
        forward_push_packed,
    },
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use ink_prelude::{
    collections::{
        BTreeMap as StdBTreeMap,
        BTreeSet as StdBTreeSet,
        BinaryHeap as StdBinaryHeap,
        LinkedList as StdLinkedList,
        VecDeque as StdVecDeque,
    },
    vec::Vec,
};
use ink_primitives::Key;

impl<K, V> SpreadLayout for StdBTreeMap<K, V>
where
    K: PackedLayout + Ord,
    V: PackedLayout,
{
    const FOOTPRINT: u64 = 1;
    const REQUIRES_DEEP_CLEAN_UP: bool = <V as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP;

    #[inline]
    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        forward_pull_packed::<Self>(ptr)
    }

    #[inline]
    fn push_spread(&self, ptr: &mut KeyPtr) {
        forward_push_packed::<Self>(self, ptr)
    }

    #[inline]
    fn clear_spread(&self, ptr: &mut KeyPtr) {
        forward_clear_packed::<Self>(self, ptr)
    }
}

impl<K, V> PackedLayout for StdBTreeMap<K, V>
where
    K: PackedLayout + Ord,
    V: PackedLayout,
{
    fn push_packed(&self, at: &Key) {
        for (key, val) in self {
            <K as PackedLayout>::push_packed(key, at);
            <V as PackedLayout>::push_packed(val, at);
        }
    }

    fn clear_packed(&self, at: &Key) {
        for (key, val) in self {
            <K as PackedLayout>::clear_packed(key, at);
            <V as PackedLayout>::clear_packed(val, at);
        }
    }

    fn pull_packed(&mut self, at: &Key) {
        // We cannot mutate keys in a map so we can forward pull signals
        // only to the values of a map.
        for val in self.values_mut() {
            <V as PackedLayout>::pull_packed(val, at);
        }
    }
}

impl<T> SpreadLayout for StdBTreeSet<T>
where
    T: PackedLayout + Ord,
{
    const FOOTPRINT: u64 = 1;
    const REQUIRES_DEEP_CLEAN_UP: bool = <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP;

    #[inline]
    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        forward_pull_packed::<Self>(ptr)
    }

    #[inline]
    fn push_spread(&self, ptr: &mut KeyPtr) {
        forward_push_packed::<Self>(self, ptr)
    }

    #[inline]
    fn clear_spread(&self, ptr: &mut KeyPtr) {
        forward_clear_packed::<Self>(self, ptr)
    }
}

impl<T> PackedLayout for StdBTreeSet<T>
where
    T: PackedLayout + Ord,
{
    fn push_packed(&self, at: &Key) {
        for key in self {
            <T as PackedLayout>::push_packed(key, at);
        }
    }

    fn clear_packed(&self, at: &Key) {
        for key in self {
            <T as PackedLayout>::clear_packed(key, at);
        }
    }

    #[inline(always)]
    fn pull_packed(&mut self, _at: &Key) {
        // We cannot mutate keys in a set so we cannot forward pull signals.
    }
}

impl<T> SpreadLayout for StdBinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    const FOOTPRINT: u64 = 1;
    const REQUIRES_DEEP_CLEAN_UP: bool = <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP;

    #[inline]
    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        forward_pull_packed::<Self>(ptr)
    }

    #[inline]
    fn push_spread(&self, ptr: &mut KeyPtr) {
        forward_push_packed::<Self>(self, ptr)
    }

    #[inline]
    fn clear_spread(&self, ptr: &mut KeyPtr) {
        forward_clear_packed::<Self>(self, ptr)
    }
}

impl<T> PackedLayout for StdBinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    fn push_packed(&self, at: &Key) {
        for value in self {
            <T as PackedLayout>::push_packed(value, at);
        }
    }

    fn clear_packed(&self, at: &Key) {
        for value in self {
            <T as PackedLayout>::clear_packed(value, at);
        }
    }

    #[inline(always)]
    fn pull_packed(&mut self, _at: &Key) {
        // We cannot mutate keys in a heap so we cannot forward pull signals.
    }
}

macro_rules! impl_push_at_for_collection {
    ( $($collection:ident),* $(,)? ) => {
        $(
            impl_always_packed_layout!($collection<T>, deep: <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP);

            impl<T> PackedLayout for $collection<T>
            where
                T: PackedLayout,
            {
                fn push_packed(&self, at: &Key) {
                    for elem in self {
                        <T as PackedLayout>::push_packed(elem, at)
                    }
                }

                fn clear_packed(&self, at: &Key) {
                    for elem in self {
                        <T as PackedLayout>::clear_packed(elem, at)
                    }
                }

                fn pull_packed(&mut self, at: &Key) {
                    for elem in self {
                        <T as PackedLayout>::pull_packed(elem, at)
                    }
                }
            }
        )*
    };
}
impl_push_at_for_collection!(Vec, StdLinkedList, StdVecDeque,);
