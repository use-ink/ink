// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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
use scale::{
    Decode,
    Encode,
};

impl_always_storage_type!(Vec<T: Encode + Decode>);
impl_always_storage_type!(StdBTreeMap<K: Ord + Encode + Decode, V: Encode + Decode>);
impl_always_storage_type!(StdLinkedList<T: Encode + Decode>);
impl_always_storage_type!(StdBinaryHeap<T: Ord + Encode + Decode>);
impl_always_storage_type!(StdBTreeSet<T: Ord + Encode + Decode>);
impl_always_storage_type!(StdVecDeque<T: Encode + Decode>);
