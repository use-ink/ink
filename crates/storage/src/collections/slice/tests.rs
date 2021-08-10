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

use crate::{
    collections::slice::{
        Slice,
        SliceMut,
    },
    lazy::LazyArray,
    traits::PackedLayout,
};
use core::convert::TryInto;

fn slice<T, const N: usize>(items: [T; N]) -> Slice<LazyArray<T, N>>
where
    T: PackedLayout + Clone,
{
    assert!(N <= u32::MAX.try_into().unwrap());
    let mut arr = LazyArray::new();
    for (i, item) in items.iter().enumerate() {
        arr.put(i as u32, Some(item.clone()))
    }
    Slice::new(0..(N as u32), arr)
}

fn slice_mut<T, const N: usize>(items: [T; N]) -> SliceMut<LazyArray<T, N>>
where
    T: PackedLayout + Clone,
{
    assert!(N <= u32::MAX.try_into().unwrap());
    let mut arr = LazyArray::new();
    for (i, item) in items.iter().enumerate() {
        arr.put(i as u32, Some(item.clone()))
    }
    // SAFETY: No other slices hold a reference to the `LazyArray.
    unsafe { SliceMut::new(0..(N as u32), arr) }
}

#[test]
fn test_iterator_nth() {
    let v = slice([0, 1, 2, 3, 4]);
    for i in 0..v.len() {
        assert_eq!(v.iter().nth(i as usize).unwrap(), &v[i]);
    }
    assert_eq!(v.iter().nth(v.len() as usize), None);

    let mut iter = v.iter();
    assert_eq!(iter.nth(2).unwrap(), &v[2]);
    assert_eq!(iter.nth(1).unwrap(), &v[4]);
}

#[test]
fn test_iterator_nth_back() {
    let v = slice([0, 1, 2, 3, 4]);
    for i in 0..v.len() {
        assert_eq!(v.iter().nth_back(i as usize).unwrap(), &v[v.len() - i - 1]);
    }
    assert_eq!(v.iter().nth_back(v.len() as usize), None);

    let mut iter = v.iter();
    assert_eq!(iter.nth_back(2).unwrap(), &v[2]);
    assert_eq!(iter.nth_back(1).unwrap(), &v[0]);
}

#[test]
fn test_iterator_last() {
    let v = slice([0, 1, 2, 3, 4]);
    assert_eq!(v.iter().last().unwrap(), &4);
    let (v, _) = v.split_at(1);
    assert_eq!(v.iter().next().unwrap(), &0);
}

#[test]
fn test_iterator_count() {
    let v = slice([0, 1, 2, 3, 4]);
    assert_eq!(v.iter().count(), 5);

    let mut iter2 = v.iter();
    iter2.next();
    iter2.next();
    assert_eq!(iter2.count(), 3);
}

#[test]
fn test_find_rfind() {
    let v = slice([0, 1, 2, 3, 4, 5]);
    let mut iter = v.iter();
    let mut i = v.len();
    while let Some(&elt) = iter.rfind(|_| true) {
        i -= 1;
        assert_eq!(elt, v[i]);
    }
    assert_eq!(i, 0);
    assert_eq!(v.iter().rfind(|&&x| x <= 3), Some(&3));
}

#[test]
fn test_iter_folds() {
    let a = slice([1, 2, 3, 4, 5]); // len>4 so the unroll is used
    assert_eq!(a.iter().fold(0, |acc, &x| 2 * acc + x), 57);
    assert_eq!(a.iter().rfold(0, |acc, &x| 2 * acc + x), 129);
    let fold = |acc: i32, &x| acc.checked_mul(2)?.checked_add(x);
    assert_eq!(a.iter().try_fold(0, &fold), Some(57));
    assert_eq!(a.iter().try_rfold(0, &fold), Some(129));

    // short-circuiting try_fold, through other methods
    let a = [0, 1, 2, 3, 5, 5, 5, 7, 8, 9];
    let mut iter = a.iter();
    assert_eq!(iter.position(|&x| x == 3), Some(3));
    assert_eq!(iter.rfind(|&&x| x == 5), Some(&5));
    assert_eq!(iter.len(), 2);
}

#[test]
fn test_iterator_nth_mut() {
    let mut v = slice_mut([0, 1, 2, 3, 4]);
    let len = v.len() as usize;

    for i in 0..v.len() {
        assert_eq!(v.iter_mut().nth(i as usize).unwrap().clone(), v[i]);
    }
    assert_eq!(v.iter_mut().nth(len), None);

    let two = v[2].clone();
    let four = v[4].clone();
    let mut iter = v.iter_mut();
    assert_eq!(iter.nth(2).unwrap().clone(), two);
    assert_eq!(iter.nth(1).unwrap().clone(), four);
}

#[test]
fn test_iterator_nth_back_mut() {
    let mut v = slice_mut([0, 1, 2, 3, 4]);
    let len = v.len() as usize;
    for i in 0..len {
        assert_eq!(
            v.iter_mut().nth_back(i).unwrap().clone(),
            v[(len - i - 1) as u32]
        );
    }
    assert_eq!(v.iter_mut().nth_back(len), None);

    let two = v[2].clone();
    let four = v[0].clone();
    let mut iter = v.iter_mut();
    assert_eq!(iter.nth_back(2).unwrap().clone(), two);
    assert_eq!(iter.nth_back(1).unwrap().clone(), four);
}

#[test]
fn test_iterator_last_mut() {
    let mut v = slice_mut([0, 1, 2, 3, 4]);
    assert_eq!(v.iter_mut().last().unwrap(), &4);
    let (mut v, _) = v.split_at_mut(1);
    assert_eq!(v.iter_mut().next().unwrap(), &0);
}

#[test]
fn test_iterator_count_mut() {
    let mut v = slice_mut([0, 1, 2, 3, 4]);
    assert_eq!(v.iter_mut().count(), 5);

    let mut iter2 = v.iter_mut();
    iter2.next();
    iter2.next();
    assert_eq!(iter2.count(), 3);
}

#[test]
fn test_find_rfind_mut() {
    let mut v = slice_mut([0, 1, 2, 3, 4, 5]);
    let mut iter = v.iter_mut();
    let v = slice_mut([0, 1, 2, 3, 4, 5]);
    let mut i = v.len();
    while let Some(elt) = iter.rfind(|_| true) {
        i -= 1;
        assert_eq!(elt.clone(), v[i]);
    }
    assert_eq!(i, 0);
    assert_eq!(v.iter().rfind(|&&x| x <= 3), Some(&3));
}

#[test]
fn test_iter_folds_mut() {
    let mut a = slice_mut([1, 2, 3, 4, 5]); // len>4 so the unroll is used
    {
        assert_eq!(a.iter_mut().fold(0, |acc, &mut x| 2 * acc + x), 57);
    }

    {
        assert_eq!(a.iter_mut().rfold(0, |acc, &mut x| 2 * acc + x), 129);
    }
    {
        let fold = |acc: i32, &mut x| acc.checked_mul(2)?.checked_add(x);
        assert_eq!(a.iter_mut().try_fold(0, &fold), Some(57));
    }
    {
        let fold = |acc: i32, &mut x| acc.checked_mul(2)?.checked_add(x);
        assert_eq!(a.iter_mut().try_rfold(0, &fold), Some(129));
    }

    // short-circuiting try_fold, through other methods
    let mut a = slice_mut([0, 1, 2, 3, 5, 5, 5, 7, 8, 9]);
    let mut iter = a.iter_mut();
    assert_eq!(iter.position(|&mut x| x == 3), Some(3));
    assert_eq!(iter.rfind(|&&mut x| x == 5), Some(&mut 5));
    assert_eq!(iter.len(), 2);
}
