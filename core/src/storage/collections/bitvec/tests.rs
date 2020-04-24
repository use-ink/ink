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

use crate::storage::{
    self,
    alloc::{
        AllocateUsing,
        BumpAlloc,
        Initialize,
    },
};
use ink_primitives::Key;

/// Returns an empty storage bit vector.
fn new_empty_bitvec() -> storage::BitVec {
    unsafe {
        let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
        storage::BitVec::allocate_using(&mut alloc).initialize_into(())
    }
}

/// Iterator over the bits that are contained in the filled test bit vector.
fn filled_bits() -> impl Iterator<Item = bool> + DoubleEndedIterator {
    vec![true, false, false, true, true, false].into_iter()
}

/// Returns a storage bit vector that is initialized as `100110`.
fn new_filled_bitvec() -> storage::BitVec {
    let mut bv = new_empty_bitvec();
    for bit in filled_bits() {
        bv.push(bit)
    }
    assert_eq!(bv.len(), 6);
    bv
}

#[test]
fn is_empty() {
    let mut bv = new_empty_bitvec();
    assert_eq!(bv.is_empty(), true);
    bv.push(true);
    assert_eq!(bv.is_empty(), false);
    bv.pop();
    assert_eq!(bv.is_empty(), true);
}

#[test]
fn len() {
    const N: u32 = 5;
    let mut bv = new_empty_bitvec();
    for n in 1..=N {
        bv.push(false);
        assert_eq!(bv.len(), n);
    }
    for n in 1..=N {
        bv.pop();
        assert_eq!(bv.len(), N - n);
    }
}

#[test]
fn first() {
    assert_eq!(new_filled_bitvec().first(), Some(true))
}

#[test]
fn first_when_empty() {
    assert_eq!(new_empty_bitvec().first(), None)
}

#[test]
fn last() {
    assert_eq!(new_filled_bitvec().last(), Some(false))
}

#[test]
fn last_when_empty() {
    assert_eq!(new_empty_bitvec().last(), None)
}

#[test]
fn get_filled() {
    let filled = new_filled_bitvec(); // `100110`
    for (n, bit) in filled_bits().enumerate() {
        assert_eq!(filled.get(n as u32), Some(bit));
    }
    assert_eq!(filled.get(6), None);
}

#[test]
fn push_empty() {
    let mut empty = new_empty_bitvec();
    empty.push(true);
    assert_eq!(empty.last(), Some(true));
    assert_eq!(empty.len(), 1);
}

#[test]
fn push_filled() {
    let mut filled = new_filled_bitvec();
    let len = filled.len();
    filled.push(false);
    assert_eq!(filled.last(), Some(false));
    assert_eq!(filled.len(), len + 1);
}

#[test]
fn extend() {
    let mut filled = new_filled_bitvec();
    let arr = [true, false, true];

    let mut expected = Vec::new();
    expected.extend(filled.iter());
    expected.extend(&arr);

    filled.extend(&arr);

    assert!(filled.iter().eq(expected.iter().cloned()));
}

#[test]
fn pop_empty() {
    assert_eq!(new_empty_bitvec().pop(), None);
}

#[test]
fn pop_filled() {
    let mut filled = new_filled_bitvec();
    for bit in filled_bits().rev() {
        assert_eq!(filled.pop(), Some(bit));
    }
    assert_eq!(filled.pop(), None);
}

#[test]
#[should_panic]
fn set_empty() {
    new_empty_bitvec().set(0, true)
}

#[test]
fn set_filled() {
    let mut filled = new_filled_bitvec();
    for (n, bit) in filled_bits().rev().enumerate() {
        filled.set(n as u32, bit);
        assert_eq!(filled.get(n as u32), Some(bit));
    }
}

#[test]
#[should_panic]
fn set_out_of_bounds() {
    let mut filled = new_filled_bitvec();
    let len = filled.len();
    filled.set(len, true);
}

#[test]
#[should_panic]
fn flip_empty() {
    new_empty_bitvec().flip(0)
}

#[test]
#[should_panic]
fn flip_out_of_bounds() {
    let mut filled = new_filled_bitvec();
    let len = filled.len();
    filled.flip(len);
}

#[test]
fn flip_filled() {
    let mut filled = new_filled_bitvec();
    for (n, bit) in filled_bits().enumerate() {
        filled.flip(n as u32);
        assert_eq!(filled.get(n as u32), Some(!bit));
    }
}

#[test]
fn iter() {
    let filled = new_filled_bitvec();
    for (actual, expected) in filled.iter().zip(filled_bits()) {
        assert_eq!(actual, expected)
    }
}

#[test]
fn iter_backwards() {
    let filled = new_filled_bitvec();
    for (actual, expected) in filled.iter().rev().zip(filled_bits().rev()) {
        assert_eq!(actual, expected)
    }
}

#[test]
fn iter_size_hint_empty() {
    let filled = new_empty_bitvec();
    assert_eq!(filled.iter().size_hint(), (0, Some(0)));
}

#[test]
fn iter_size_hint_filled() {
    let len = filled_bits().count();
    let filled = new_filled_bitvec();
    assert_eq!(filled.iter().size_hint(), (len, Some(len)));
}

fn zero_bitvec_with_len(len: usize) -> storage::BitVec {
    let mut bv = new_empty_bitvec();
    for _ in 0..len {
        bv.push(false);
    }
    bv
}

#[test]
fn first_set_position() {
    for &n in &[
        0_u32, 1, 2, 5, 10, 31, 32, 33, 500, 1000, 1023, 1024, 2047, 2048, 2049,
    ] {
        let mut bv = zero_bitvec_with_len(3000); // Has 3 bit blocks.
        bv.set(n, true);
        assert_eq!(bv.first_set_position(), Some(n));
    }
}

#[test]
fn first_set_position_push_pop() {
    let mut bv = zero_bitvec_with_len(2000);
    bv.push(true);
    assert_eq!(bv.first_set_position(), Some(2000));
    bv.pop();
    assert_eq!(bv.first_set_position(), None);
}
