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

use super::Vec as StorageVec;
use crate::{
    test_utils::FuzzCollection,
    traits::{
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    },
    Pack,
};

use quickcheck::{
    Arbitrary,
    Gen,
};
use std::{
    iter::FromIterator,
    vec::Vec,
};

impl<T> Arbitrary for StorageVec<T>
where
    T: Arbitrary + PackedLayout + Send + Clone + 'static,
{
    fn arbitrary(g: &mut Gen) -> StorageVec<T> {
        let vec = Vec::<T>::arbitrary(g);
        StorageVec::<T>::from_iter(vec)
    }
}

impl<T> Clone for StorageVec<T>
where
    T: PackedLayout + Clone,
{
    fn clone(&self) -> Self {
        let mut svec = StorageVec::<T>::new();
        self.iter().for_each(|v| svec.push(v.clone()));
        svec
    }
}

impl<'a, T> FuzzCollection for &'a mut StorageVec<T>
where
    T: Clone + PackedLayout,
{
    type Collection = StorageVec<T>;
    type Item = &'a mut T;

    /// Makes `self` equal to `instance2` by executing a series of operations
    /// on `self`.
    fn equalize(&mut self, instance2: &Self::Collection) {
        self.clear();
        instance2.into_iter().for_each(|v| self.push(v.clone()));
    }

    /// `val` is a value from the vector. We take an element out
    /// of `self` and assign it to `val`.
    ///
    /// Hence this method only might modify values of `item`, leaving
    /// others intact.
    fn assign(&mut self, val: Self::Item) {
        if let Some(popped_val) = self.pop() {
            *val = popped_val.clone();
        }
    }
}

crate::fuzz_storage!("vec_1", StorageVec<u32>);
crate::fuzz_storage!("vec_2", StorageVec<Option<Pack<Option<u32>>>>);
crate::fuzz_storage!("vec_3", StorageVec<(bool, (u32, u128))>);
crate::fuzz_storage!("vec_4", StorageVec<(i128, u32, bool, Option<(u32, i128)>)>);
