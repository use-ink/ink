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

use crate::{
    env,
    env::Result,
    storage::{
        alloc::{
            AllocateUsing,
            BumpAlloc,
            Initialize,
        },
        Stash,
    },
};
use ink_primitives::Key;

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
fn new_unchecked() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let stash = empty_stash();
        // Initial invariant.
        assert_eq!(stash.len(), 0);
        assert!(stash.is_empty());
        assert_eq!(stash.iter().next(), None);
        Ok(())
    })
}

#[test]
fn put_empty() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut stash = empty_stash();
        // Before and after first put.
        assert_eq!(stash.get(0), None);
        assert_eq!(stash.put(42), 0);
        assert_eq!(stash.get(0), Some(&42));
        Ok(())
    })
}

#[test]
fn put_filled() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
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
        Ok(())
    })
}

#[test]
fn take_empty() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut stash = empty_stash();
        assert_eq!(stash.take(0), None);
        assert_eq!(stash.take(1000), None);
        Ok(())
    })
}

#[test]
fn take_filled() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
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
        Ok(())
    })
}

#[test]
fn put_take() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
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
        Ok(())
    })
}

#[test]
fn iter() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let stash = filled_stash();
        let mut iter = stash.iter();
        assert_eq!(iter.next(), Some((0, &5)));
        assert_eq!(iter.next(), Some((1, &42)));
        assert_eq!(iter.next(), Some((2, &1337)));
        assert_eq!(iter.next(), Some((3, &77)));
        assert_eq!(iter.next(), None);
        Ok(())
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
fn iter_holey() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let stash = holey_stash();
        let mut iter = stash.iter();
        assert_eq!(iter.next(), Some((0, &5)));
        assert_eq!(iter.next(), Some((2, &1337)));
        assert_eq!(iter.next(), Some((4, &123)));
        assert_eq!(iter.next(), None);
        Ok(())
    })
}

#[test]
fn iter_back() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let stash = filled_stash();
        let mut iter = stash.iter();
        assert_eq!(iter.next_back(), Some((3, &77)));
        assert_eq!(iter.next_back(), Some((2, &1337)));
        assert_eq!(iter.next_back(), Some((1, &42)));
        assert_eq!(iter.next_back(), Some((0, &5)));
        assert_eq!(iter.next_back(), None);
        Ok(())
    })
}

#[test]
fn iter_back_holey() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let stash = holey_stash();
        let mut iter = stash.iter();
        assert_eq!(iter.next_back(), Some((4, &123)));
        assert_eq!(iter.next_back(), Some((2, &1337)));
        assert_eq!(iter.next_back(), Some((0, &5)));
        assert_eq!(iter.next_back(), None);
        Ok(())
    })
}

#[test]
fn iter_size_hint() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let stash = filled_stash();
        let mut iter = stash.iter();
        assert_eq!(iter.size_hint(), (4, Some(4)));
        iter.next();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        Ok(())
    })
}
