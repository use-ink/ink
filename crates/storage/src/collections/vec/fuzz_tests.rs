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

// The fuzz tests are testing complex types.
#![allow(clippy::type_complexity)]

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
use itertools::Itertools;
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
            *val = popped_val;
        }
    }
}

crate::fuzz_storage!("vec_1", StorageVec<u32>);
crate::fuzz_storage!("vec_2", StorageVec<Option<Pack<Option<u32>>>>);
crate::fuzz_storage!("vec_3", StorageVec<(bool, (u32, u128))>);
crate::fuzz_storage!("vec_4", StorageVec<(i128, u32, bool, Option<(u32, i128)>)>);

#[quickcheck]
fn fuzz_binary_search(mut std_vec: Vec<i32>) {
    // given
    if std_vec.is_empty() {
        return
    }
    let original_std_vec = std_vec.clone();
    std_vec.sort_unstable();
    let ink_vec = StorageVec::from_iter(std_vec.clone());

    for x in original_std_vec {
        // when
        let index = std_vec
            .binary_search(&x)
            .expect("`x` must be found in `Vec`") as u32;
        let ink_index = ink_vec
            .binary_search(&x)
            .expect("`x` must be found in `StorageVec`");
        let ink_index_by = ink_vec
            .binary_search_by(|by_x| by_x.cmp(&x))
            .expect("`x` must be found in `StorageVec`");

        // then
        assert_eq!(index, ink_index);
        assert_eq!(index, ink_index_by);
    }
}

#[quickcheck]
fn fuzz_binary_search_nonexistent(std_vec: Vec<i32>) {
    // given
    if std_vec.is_empty() {
        return
    }
    let mut unique_std_vec: Vec<i32> = std_vec.into_iter().unique().collect();
    let removed_el = unique_std_vec
        .pop()
        .expect("length is non-zero, first element must exist");
    unique_std_vec.sort_unstable();
    let ink_vec = StorageVec::from_iter(unique_std_vec.clone());

    // when
    let std_err_index = unique_std_vec
        .binary_search(&removed_el)
        .expect_err("element must not be found") as u32;
    let ink_err_index = ink_vec
        .binary_search(&removed_el)
        .expect_err("element must not be found");
    let ink_search_by_err_index = ink_vec
        .binary_search_by(|by_x| by_x.cmp(&removed_el))
        .expect_err("element must not be found");

    // then
    assert_eq!(std_err_index, ink_err_index);
    assert_eq!(std_err_index, ink_search_by_err_index);
}

#[quickcheck]
fn fuzz_binary_search_by_key(mut std_vec: Vec<(i32, i32)>) {
    // given
    if std_vec.is_empty() {
        return
    }
    let original_std_vec = std_vec.clone();
    std_vec.sort_by_key(|&(_a, b)| b);
    let ink_vec = StorageVec::from_iter(std_vec.clone());

    for (_x, y) in original_std_vec {
        // when
        let std_index = std_vec
            .binary_search_by_key(&y, |&(_a, b)| b)
            .expect("`y` must be found in `Vec`") as u32;
        let ink_index = ink_vec
            .binary_search_by_key(&y, |&(_a, b)| b)
            .expect("`y` must be found in `StorageVec`");

        // then
        assert_eq!(std_index, ink_index);
    }
}
#[quickcheck]
fn fuzz_binary_search_by_key_nonexistent(std_vec: Vec<(i32, i32)>) {
    // given
    if std_vec.is_empty() {
        return
    }
    let mut unique_std_vec: Vec<(i32, i32)> =
        std_vec.into_iter().unique_by(|&(_a, b)| b).collect();
    let removed_el = unique_std_vec
        .pop()
        .expect("length is non-zero, first element must exist");
    unique_std_vec.sort_by_key(|&(_a, b)| b);
    let ink_vec = StorageVec::from_iter(unique_std_vec.clone());

    // when
    let std_err_index = unique_std_vec
        .binary_search_by_key(&removed_el.1, |&(_a, b)| b)
        .expect_err("element must not be found in `Vec`") as u32;
    let ink_err_index = ink_vec
        .binary_search_by_key(&removed_el.1, |&(_a, b)| b)
        .expect_err("element must not be found in `StorageVec`");

    // then
    assert_eq!(std_err_index, ink_err_index);
}
