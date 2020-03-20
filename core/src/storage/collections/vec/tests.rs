// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use super::*;
use crate::storage::{
    self,
    alloc::{
        AllocateUsing,
        BumpAlloc,
        Initialize,
    },
};
use ink_primitives::Key;

/// Returns an empty storage vector at address `0x42`.
fn new_empty_vec<T>() -> storage::Vec<T> {
    unsafe {
        let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
        Vec::<T>::allocate_using(&mut alloc).initialize_into(())
    }
}

/// Returns a filled storage vector at address `0x42`.
///
/// Elements are `[5, 42, 1337, 77]` in that order.
fn new_filled_vec() -> storage::Vec<i32> {
    let mut vec = new_empty_vec();
    vec.push(5);
    vec.push(42);
    vec.push(1337);
    vec.push(77);
    assert_eq!(vec.len(), 4);
    vec
}

#[test]
fn init() {
    let vec = new_empty_vec::<i32>();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.is_empty(), true);
    assert_eq!(vec.iter().next(), None);
}

#[test]
fn simple() {
    let mut vec = new_empty_vec();
    assert_eq!(vec.len(), 0);
    vec.push(5);
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.get(0), Some(&5));
    {
        let mut iter = vec.iter();
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), None);
    }
    assert_eq!(vec.pop(), Some(5));
    assert_eq!(vec.len(), 0);
}

#[test]
fn pop_empty() {
    let mut vec = new_empty_vec::<i32>();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.pop(), None);
    assert_eq!(vec.len(), 0);
}

#[test]
fn iter() {
    let vec = new_filled_vec();
    let mut iter = vec.iter();
    assert_eq!(iter.next(), Some(&5));
    assert_eq!(iter.next(), Some(&42));
    assert_eq!(iter.next(), Some(&1337));
    assert_eq!(iter.next(), Some(&77));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_back() {
    let vec = new_filled_vec();
    let mut iter = vec.iter();
    assert_eq!(iter.next_back(), Some(&77));
    assert_eq!(iter.next_back(), Some(&1337));
    assert_eq!(iter.next_back(), Some(&42));
    assert_eq!(iter.next_back(), Some(&5));
    assert_eq!(iter.next_back(), None);
}

#[test]
fn get() {
    let vec = new_filled_vec();
    assert_eq!(vec.get(0), Some(&5));
    assert_eq!(vec.get(1), Some(&42));
    assert_eq!(vec.get(2), Some(&1337));
    assert_eq!(vec.get(3), Some(&77));
    assert_eq!(vec.get(4), None);
    assert_eq!(vec.get(u32::max_value()), None);
}

#[test]
fn index() {
    let vec = new_filled_vec();
    assert_eq!(vec[0], 5);
    assert_eq!(vec[1], 42);
    assert_eq!(vec[2], 1337);
    assert_eq!(vec[3], 77);
}

#[test]
fn index_mut() {
    let mut vec = {
        let mut vec = new_empty_vec();
        vec.push(String::from("Hello"));
        vec.push(String::from(", "));
        vec.push(String::from("World!"));
        assert_eq!(vec.len(), 3);
        vec
    };
    vec[2] = String::from("Substrate!");
    assert_eq!(vec[0], "Hello");
    assert_eq!(vec[1], ", ");
    assert_eq!(vec[2], "Substrate!");
}

#[test]
fn index_comp() {
    let vec = {
        let mut vec = new_empty_vec();
        vec.push(String::from("Hello"));
        vec.push(String::from(", "));
        vec.push(String::from("World!"));
        assert_eq!(vec.len(), 3);
        vec
    };
    assert_eq!(vec[0], "Hello");
}

#[test]
#[should_panic]
fn index_fail_0() {
    let vec = new_filled_vec();
    let _ = vec[4];
}

#[test]
#[should_panic]
fn index_fail_1() {
    let vec = new_filled_vec();
    let _ = vec[u32::max_value()];
}

#[test]
#[should_panic]
fn index_fail_2() {
    let vec = new_empty_vec::<i32>();
    let _ = vec[0];
}

#[test]
fn mutate() {
    let mut vec = new_filled_vec();
    assert_eq!(vec.mutate(0, |x| *x += 10), Some(&15));
    assert_eq!(vec.mutate(1, |x| *x *= 2), Some(&84));
    assert_eq!(vec.mutate(4, |x| *x *= 2), None);
    assert_eq!(vec.mutate(u32::max_value(), |_| ()), None);
}

#[test]
fn replace() {
    let mut vec = new_filled_vec();
    assert_eq!(vec.replace(0, || 1), Some(5));
    assert_eq!(vec.get(0), Some(&1));
    assert_eq!(vec.replace(1, || 50), Some(42));
    assert_eq!(vec.get(1), Some(&50));
    assert_eq!(vec.replace(4, || 999), None);
    assert_eq!(vec.get(4), None);
}

#[test]
fn swap() {
    let mut vec = new_filled_vec();
    assert_eq!(vec.get(1), Some(&42));
    assert_eq!(vec.get(3), Some(&77));
    vec.swap(1, 3);
    assert_eq!(vec.get(1), Some(&77));
    assert_eq!(vec.get(3), Some(&42));
}

#[test]
fn swap_same() {
    let mut vec = new_filled_vec();
    assert_eq!(vec.get(2), Some(&1337));
    // Does basically nothing.
    vec.swap(2, 2);
    assert_eq!(vec.get(2), Some(&1337));
}

#[test]
#[should_panic]
fn swap_invalid() {
    let mut vec = new_filled_vec();
    vec.swap(0, u32::max_value());
}

#[test]
fn swap_remove() {
    let mut vec = new_filled_vec();
    assert_eq!(vec.get(1), Some(&42));
    assert_eq!(vec.get(3), Some(&77));
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.swap_remove(1), Some(42));
    assert_eq!(vec.get(1), Some(&77));
    assert_eq!(vec.get(3), None);
    assert_eq!(vec.len(), 3);
}

#[test]
fn swap_remove_empty() {
    let mut vec = new_empty_vec::<i32>();
    assert_eq!(vec.swap_remove(0), None);
}

#[test]
fn iter_size_hint() {
    let vec = new_filled_vec();
    let mut iter = vec.iter();
    assert_eq!(iter.size_hint(), (4, Some(4)));
    let _ = iter.next();
    assert_eq!(iter.size_hint(), (3, Some(3)));
}

#[test]
fn extend() {
    let mut vec1 = new_filled_vec();
    let arr = [1, 2, 3];

    let mut expected = ink_prelude::vec::Vec::new();
    expected.extend(vec1.iter());
    expected.extend(&arr);

    vec1.extend(&arr);

    assert!(vec1.iter().eq(expected.iter()));
}

#[test]
fn regression_issue_193() {
    let mut vec = new_empty_vec();
    vec.push(5);
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.swap_remove(0), Some(5));
    assert_eq!(vec.len(), 0);
}
