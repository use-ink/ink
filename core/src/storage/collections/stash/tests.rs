// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    storage::{
        alloc::{
            AllocateUsing,
            BumpAlloc,
            Initialize,
        },
        Key,
        Stash,
    },
    test_utils::run_test,
};

fn empty_stash() -> Stash<i32> {
    unsafe {
        let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
        Stash::allocate_using(&mut alloc).initialize_into(())
    }
}

fn filled_stash() -> Stash<i32> {
    let mut stash = empty_stash();
    stash.put(5);
    stash.put(42);
    stash.put(1337);
    stash.put(77);
    assert_eq!(stash.len(), 4);
    stash
}

#[test]
fn new_unchecked() {
    run_test(|| {
        let stash = empty_stash();
        // Initial invariant.
        assert_eq!(stash.len(), 0);
        assert!(stash.is_empty());
        assert_eq!(stash.iter().next(), None);
    })
}

#[test]
fn put_empty() {
    run_test(|| {
        let mut stash = empty_stash();
        // Before and after first put.
        assert_eq!(stash.get(0), None);
        assert_eq!(stash.put(42), 0);
        assert_eq!(stash.get(0), Some(&42));
    })
}

#[test]
fn put_filled() {
    run_test(|| {
        let mut stash = filled_stash();
        // Before and next put.
        assert_eq!(stash.get(0), Some(&5));
        assert_eq!(stash.get(1), Some(&42));
        assert_eq!(stash.get(2), Some(&1337));
        assert_eq!(stash.get(3), Some(&77));
        assert_eq!(stash.get(4), None);
        assert_eq!(stash.len(), 4);
        // Now put.
        assert_eq!(stash.put(123), 4);
        assert_eq!(stash.get(4), Some(&123));
        assert_eq!(stash.len(), 5);
    })
}

#[test]
fn take_empty() {
    run_test(|| {
        let mut stash = empty_stash();
        assert_eq!(stash.take(0), None);
        assert_eq!(stash.take(1000), None);
    })
}

#[test]
fn take_filled() {
    run_test(|| {
        let mut stash = filled_stash();
        // Take and check len
        assert_eq!(stash.len(), 4);
        assert_eq!(stash.take(0), Some(5));
        assert_eq!(stash.len(), 3);
        assert_eq!(stash.take(1), Some(42));
        assert_eq!(stash.len(), 2);
        assert_eq!(stash.take(2), Some(1337));
        assert_eq!(stash.len(), 1);
        assert_eq!(stash.take(3), Some(77));
        assert_eq!(stash.len(), 0);
        assert_eq!(stash.take(4), None);
        assert_eq!(stash.len(), 0);
    })
}

#[test]
fn put_take() {
    run_test(|| {
        let mut stash = filled_stash();
        // Take and put "randomly"
        //
        // Layout of the stash in memory:
        //
        //          |----------------------------------|
        // Index    |    0 |    1 |    2 |    3 |    4 |
        //          |------|------|------|------|------|
        // Occupied |    5 |   42 | 1337 |   77 |      |
        // Vacant   |      |      |      |      |      |
        //          |----------------------------------|
        // next_vacant = 4
        assert_eq!(stash.take(2), Some(1337));
        //          |----------------------------------|
        // Index    |    0 |    1 |    2 |    3 |    4 |
        //          |------|------|------|------|------|
        // Occupied |    5 |   42 |      |   77 |      |
        // Vacant   |      |      |    4 |      |      |
        //          |----------------------------------|
        // next_vacant = 2
        assert_eq!(stash.take(0), Some(5));
        //          |----------------------------------|
        // Index    |    0 |    1 |    2 |    3 |    4 |
        //          |------|------|------|------|------|
        // Occupied |      |   42 |      |   77 |      |
        // Vacant   |    2 |      |    4 |      |      |
        //          |----------------------------------|
        // next_vacant = 0
        assert_eq!(stash.put(123), 0);
        //          |----------------------------------|
        // Index    |    0 |    1 |    2 |    3 |    4 |
        //          |------|------|------|------|------|
        // Occupied |  123 |   42 |      |   77 |      |
        // Vacant   |      |      |    4 |      |      |
        //          |----------------------------------|
        // next_vacant = 2
        assert_eq!(stash.put(555), 2);
        //          |----------------------------------|
        // Index    |    0 |    1 |    2 |    3 |    4 |
        //          |------|------|------|------|------|
        // Occupied |  123 |   42 |  555 |   77 |      |
        // Vacant   |      |      |      |      |      |
        //          |----------------------------------|
        // next_vacant = 4
        assert_eq!(stash.put(999), 4);
        //          |------------------------------------------
        // Index    |    0 |    1 |    2 |    3 |    4 |    5 |
        //          |------|------|------|------|------|------|
        // Occupied |  123 |   42 |  555 |   77 |  999 |      |
        // Vacant   |      |      |      |      |      |      |
        //          |------------------------------------------
        // next_vacant = 5
        assert_eq!(stash.take(3), Some(77));
        //          |------------------------------------------
        // Index    |    0 |    1 |    2 |    3 |    4 |    5 |
        //          |------|------|------|------|------|------|
        // Occupied |  123 |   42 |  555 |      |  999 |      |
        // Vacant   |      |      |      |    5 |      |      |
        //          |------------------------------------------
        // next_vacant = 3
        assert_eq!(stash.take(0), Some(123));
        //          |------------------------------------------
        // Index    |    0 |    1 |    2 |    3 |    4 |    5 |
        //          |------|------|------|------|------|------|
        // Occupied |      |   42 |  555 |      |  999 |      |
        // Vacant   |    3 |      |      |    5 |      |      |
        //          |------------------------------------------
        // next_vacant = 0
        assert_eq!(stash.put(911), 0);
        //          |------------------------------------------
        // Index    |    0 |    1 |    2 |    3 |    4 |    5 |
        //          |------|------|------|------|------|------|
        // Occupied |  911 |   42 |  555 |      |  999 |      |
        // Vacant   |      |      |      |    5 |      |      |
        //          |------------------------------------------
        // next_vacant = 3
        assert_eq!(stash.take(3), None);
        assert_eq!(stash.take(1), Some(42));
        //          |------------------------------------------
        // Index    |    0 |    1 |    2 |    3 |    4 |    5 |
        //          |------|------|------|------|------|------|
        // Occupied |  911 |      |  555 |      |  999 |      |
        // Vacant   |      |    3 |      |    5 |      |      |
        //          |------------------------------------------
        // next_vacant = 1
    })
}

#[test]
fn iter() {
    run_test(|| {
        let stash = filled_stash();
        let mut iter = stash.iter();
        assert_eq!(iter.next(), Some((0, &5)));
        assert_eq!(iter.next(), Some((1, &42)));
        assert_eq!(iter.next(), Some((2, &1337)));
        assert_eq!(iter.next(), Some((3, &77)));
        assert_eq!(iter.next(), None);
    })
}

fn holey_stash() -> Stash<i32> {
    let mut stash = filled_stash();
    stash.put(123);
    stash.take(1);
    stash.take(3);
    stash
}

#[test]
fn iter_holey() {
    run_test(|| {
        let stash = holey_stash();
        let mut iter = stash.iter();
        assert_eq!(iter.next(), Some((0, &5)));
        assert_eq!(iter.next(), Some((2, &1337)));
        assert_eq!(iter.next(), Some((4, &123)));
        assert_eq!(iter.next(), None);
    })
}

#[test]
fn iter_back() {
    run_test(|| {
        let stash = filled_stash();
        let mut iter = stash.iter();
        assert_eq!(iter.next_back(), Some((3, &77)));
        assert_eq!(iter.next_back(), Some((2, &1337)));
        assert_eq!(iter.next_back(), Some((1, &42)));
        assert_eq!(iter.next_back(), Some((0, &5)));
        assert_eq!(iter.next_back(), None);
    })
}

#[test]
fn iter_back_holey() {
    run_test(|| {
        let stash = holey_stash();
        let mut iter = stash.iter();
        assert_eq!(iter.next_back(), Some((4, &123)));
        assert_eq!(iter.next_back(), Some((2, &1337)));
        assert_eq!(iter.next_back(), Some((0, &5)));
        assert_eq!(iter.next_back(), None);
    })
}

#[test]
fn iter_size_hint() {
    run_test(|| {
        let stash = filled_stash();
        let mut iter = stash.iter();
        assert_eq!(iter.size_hint(), (4, Some(4)));
        iter.next();
        assert_eq!(iter.size_hint(), (3, Some(3)));
    })
}
