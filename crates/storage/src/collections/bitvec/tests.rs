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

use super::Bitvec as StorageBitvec;
use crate::traits::{
    KeyPtr,
    SpreadLayout,
};
use ink_primitives::Key;

#[test]
fn new_default_works() {
    // Check if `Bitvec::new` works:
    let mut bitvec = StorageBitvec::new();
    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.capacity(), 0);
    assert!(bitvec.is_empty());
    assert_eq!(bitvec.bits().next(), None);
    assert_eq!(bitvec.get(0), None);
    assert!(bitvec.first().is_none());
    assert!(bitvec.first_mut().is_none());
    assert!(bitvec.last().is_none());
    assert!(bitvec.last_mut().is_none());
    // Check if `Bitvec::default` works:
    let mut default = StorageBitvec::default();
    assert_eq!(default.len(), 0);
    assert_eq!(bitvec.capacity(), 0);
    assert!(default.is_empty());
    assert_eq!(default.bits().next(), None);
    assert_eq!(default.get(0), None);
    assert!(default.first().is_none());
    assert!(default.first_mut().is_none());
    assert!(default.last().is_none());
    assert!(default.last_mut().is_none());
    // Check if both are equal:
    assert_eq!(bitvec, default);
}

/// Creates a storage bitvector where every bit at every 5th and 13th index
/// is set to `1` (true). The bitvector has a total length of 600 bits which
/// requires it to have 3 chunks of 256-bit giving a capacity of 768 bits.
fn bitvec_600() -> StorageBitvec {
    let bitvec = (0..600)
        .map(|i| (i % 5) == 0 || (i % 13) == 0)
        .collect::<StorageBitvec>();
    assert_eq!(bitvec.len(), 600);
    assert_eq!(bitvec.capacity(), 768);
    bitvec
}

#[test]
fn get_works() {
    let mut bitvec = bitvec_600();
    for i in 0..bitvec.len() {
        assert_eq!(bitvec.get(i), Some((i % 5) == 0 || (i % 13) == 0));
        assert_eq!(
            bitvec.get_mut(i).map(|b| b.get()),
            Some((i % 5) == 0 || (i % 13) == 0)
        );
    }
}

#[test]
fn iter_next_works() {
    let bitvec = bitvec_600();
    // Test iterator over read-only bits.
    for (i, bit) in bitvec.bits().enumerate() {
        assert_eq!(bit, (i % 5) == 0 || (i % 13) == 0);
    }
    // Test iterator over mutable accessors to bits.
    let mut bitvec = bitvec;
    for (i, accessor) in bitvec.bits_mut().enumerate() {
        assert_eq!(accessor.get(), (i % 5) == 0 || (i % 13) == 0);
    }
}

#[test]
fn iter_next_back_works() {
    let bitvec = bitvec_600();
    // Test iterator over read-only bits.
    for (i, bit) in bitvec.bits().enumerate().rev() {
        assert_eq!(bit, (i % 5) == 0 || (i % 13) == 0);
    }
    // Test iterator over mutable accessors to bits.
    let mut bitvec = bitvec;
    for (i, accessor) in bitvec.bits_mut().enumerate().rev() {
        assert_eq!(accessor.get(), (i % 5) == 0 || (i % 13) == 0);
    }
}

#[test]
fn double_ended_iter_works() {
    let mut bitvec = StorageBitvec::default();
    bitvec.push(true);
    bitvec.push(true);
    bitvec.push(true);

    let mut iter = bitvec.bits();
    assert_eq!(Some(true), iter.next());
    assert_eq!(Some(true), iter.next_back());
    assert_eq!(Some(true), iter.next());
    assert_eq!(None, iter.next());
    assert_eq!(None, iter.next_back());
}

#[test]
fn push_works() {
    let mut bitvec = StorageBitvec::new();
    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.capacity(), 0);
    // Push `1`
    bitvec.push(true);
    assert_eq!(bitvec.len(), 1);
    assert_eq!(bitvec.capacity(), 256);
    assert_eq!(bitvec.first(), Some(true));
    assert_eq!(bitvec.first_mut().map(|access| access.get()), Some(true));
    assert_eq!(bitvec.last(), Some(true));
    assert_eq!(bitvec.last_mut().map(|access| access.get()), Some(true));
    // Push `0`
    bitvec.push(false);
    assert_eq!(bitvec.len(), 2);
    assert_eq!(bitvec.capacity(), 256);
    assert_eq!(bitvec.first(), Some(true));
    assert_eq!(bitvec.first_mut().map(|access| access.get()), Some(true));
    assert_eq!(bitvec.last(), Some(false));
    assert_eq!(bitvec.last_mut().map(|access| access.get()), Some(false));
    // Push `1`
    bitvec.push(true);
    assert_eq!(bitvec.len(), 3);
    assert_eq!(bitvec.capacity(), 256);
    assert_eq!(bitvec.first(), Some(true));
    assert_eq!(bitvec.first_mut().map(|access| access.get()), Some(true));
    assert_eq!(bitvec.last(), Some(true));
    assert_eq!(bitvec.last_mut().map(|access| access.get()), Some(true));
}

#[test]
fn pop_works() {
    let mut bitvec = [true, false, true].iter().collect::<StorageBitvec>();
    assert_eq!(bitvec.len(), 3);
    assert_eq!(bitvec.capacity(), 256);
    // Pop `1` (true)
    assert_eq!(bitvec.pop(), Some(true));
    assert_eq!(bitvec.len(), 2);
    assert_eq!(bitvec.capacity(), 256);
    assert_eq!(bitvec.first(), Some(true));
    assert_eq!(bitvec.first_mut().map(|access| access.get()), Some(true));
    assert_eq!(bitvec.last(), Some(false));
    assert_eq!(bitvec.last_mut().map(|access| access.get()), Some(false));
    // Pop `0` (false)
    assert_eq!(bitvec.pop(), Some(false));
    assert_eq!(bitvec.len(), 1);
    assert_eq!(bitvec.capacity(), 256);
    assert_eq!(bitvec.first(), Some(true));
    assert_eq!(bitvec.first_mut().map(|access| access.get()), Some(true));
    assert_eq!(bitvec.last(), Some(true));
    assert_eq!(bitvec.last_mut().map(|access| access.get()), Some(true));
    // Pop `1` (true)
    assert_eq!(bitvec.pop(), Some(true));
    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.capacity(), 256);
    assert!(bitvec.first().is_none());
    assert!(bitvec.first_mut().is_none());
    assert!(bitvec.last().is_none());
    assert!(bitvec.last_mut().is_none());
}

#[test]
fn spread_layout_push_pull_works() -> ink_env::Result<()> {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let bv1 = bitvec_600();
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&bv1, &mut KeyPtr::from(root_key));
        // Load the pushed storage vector into another instance and check that
        // both instances are equal:
        let bv2 =
            <StorageBitvec as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(bv1, bv2);
        Ok(())
    })
}

#[test]
#[should_panic(expected = "encountered empty storage cell")]
fn spread_layout_clear_works() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let bv1 = bitvec_600();
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&bv1, &mut KeyPtr::from(root_key));
        // It has already been asserted that a valid instance can be pulled
        // from contract storage after a push to the same storage region.
        //
        // Now clear the associated storage from `bv1` and check whether
        // loading another instance from this storage will panic since the
        // vector's length property cannot read a value:
        SpreadLayout::clear_spread(&bv1, &mut KeyPtr::from(root_key));
        let _ = <StorageBitvec as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        Ok(())
    })
    .unwrap()
}
