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
        BinaryHeap,
    },
};
use core::{
    cmp::Ord,
    fmt::Debug,
};
use ink_prelude::collections::HashSet;
use ink_primitives::Key;
use scale::{
    Codec,
    Decode,
    Encode,
};

fn empty_heap() -> BinaryHeap<i32> {
    unsafe {
        let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
        BinaryHeap::allocate_using(&mut alloc).initialize_into(())
    }
}

fn filled_heap() -> BinaryHeap<i32> {
    let mut heap = empty_heap();
    heap.push(42);
    heap.push(5);
    heap.push(1337);
    heap.push(77);
    assert_eq!(heap.len(), 4);
    heap
}

/// Pushes all element from `vec` onto the heap, in the order in which they
/// are supplied in the vector.
///
/// Subsequently all elements are popped from the vec and for the retrieved
/// elements it is asserted that they are in the exact same order as the ones
/// in `expected`. The `expected` vec must contain all elements which are
/// returned, as the function finally checks that there are no more elements
/// left in the heap.
fn assert_push_equals_sorted_pop<T: Ord + Codec + Debug>(
    heap: &mut BinaryHeap<T>,
    vec: Vec<T>,
) {
    vec.into_iter().for_each(|i| heap.push(i));

    let mut prior = None;
    while let Some(val) = heap.pop() {
        if let Some(p) = prior {
            assert!(val <= p); // it's a max heap
        }
        prior = Some(val);
    }

    assert_eq!(heap.pop(), None);
    assert_eq!(heap.len(), 0);
}

#[test]
fn new_unchecked() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let heap = empty_heap();

        // then
        assert_eq!(heap.len(), 0);
        assert!(heap.is_empty());
        assert_eq!(heap.iter().next(), None);
        Ok(())
    })
}

#[test]
fn push_on_empty_heap() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut heap = empty_heap();
        assert_eq!(heap.pop(), None);

        // when
        heap.push(42);

        // then
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.pop(), Some(42));
        Ok(())
    })
}

#[test]
fn push_duplicates_max() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut heap = empty_heap();

        // when
        heap.push(10);
        heap.push(20);
        heap.push(10);
        heap.push(20);

        // then
        assert_eq!(heap.pop(), Some(20));
        assert_eq!(heap.pop(), Some(20));
        assert_eq!(heap.pop(), Some(10));
        assert_eq!(heap.pop(), Some(10));
        Ok(())
    })
}

#[test]
fn peek() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut heap = empty_heap();
        assert_eq!(heap.peek(), None);

        // when
        heap.push(42);

        // then
        assert_eq!(heap.peek(), Some(&42));
        Ok(())
    })
}

#[test]
fn peek_mut() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut heap = empty_heap();
        heap.push(42);

        // when
        let val = heap.peek_mut().unwrap();
        assert_eq!(val, &42);
        *val = 1337;

        // then
        assert_eq!(heap.peek(), Some(&1337));
        Ok(())
    })
}

#[test]
fn pop_empty_and_refill() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut heap = filled_heap();
        for _ in 0..heap.len() {
            let _ = heap.pop();
        }
        assert_eq!(heap.len(), 0);

        // when
        heap.push(123);

        // then
        assert_eq!(heap.pop(), Some(123));
        assert_eq!(heap.len(), 0);
        Ok(())
    })
}

#[test]
fn take_empty() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut heap = empty_heap();

        // then
        assert_eq!(heap.pop(), None);
        assert_eq!(heap.peek(), None);
        assert_eq!(heap.peek_mut(), None);
        Ok(())
    })
}

#[test]
fn push_negative_positive_range_min() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut heap = empty_heap();

        // when
        heap.push(-1);
        heap.push(0);
        heap.push(1);

        // then
        assert_eq!(heap.len(), 3);
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), Some(0));
        assert_eq!(heap.pop(), Some(-1));
        Ok(())
    })
}

#[test]
fn push_negative_positive_range_max() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut heap = empty_heap();

        // when
        heap.push(-1);
        heap.push(0);
        heap.push(1);

        // then
        assert_eq!(heap.len(), 3);
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), Some(0));
        assert_eq!(heap.pop(), Some(-1));
        Ok(())
    })
}

#[test]
fn iter() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let heap = filled_heap();

        // when
        let mut iter = heap.iter();

        // then
        // order can be arbitrary
        assert_eq!(iter.next(), Some((0, &1337)));
        assert_eq!(iter.next(), Some((1, &77)));
        assert_eq!(iter.next(), Some((2, &42)));
        assert_eq!(iter.next(), Some((3, &5)));
        assert_eq!(iter.next(), None);
        Ok(())
    })
}

#[test]
fn iter_back() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let heap = filled_heap();

        // when
        let mut iter = heap.iter();

        // then
        assert_eq!(iter.next_back(), Some((3, &5)));
        assert_eq!(iter.next_back(), Some((2, &42)));
        assert_eq!(iter.next_back(), Some((1, &77)));
        assert_eq!(iter.next_back(), Some((0, &1337)));
        assert_eq!(iter.next_back(), None);
        Ok(())
    })
}

#[test]
fn iter_size_hint() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let heap = filled_heap();

        // when
        let mut iter = heap.iter();
        assert_eq!(iter.size_hint(), (4, Some(4)));

        // then
        iter.next();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        Ok(())
    })
}

#[test]
fn unordered_push_results_in_ordered_pop() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut heap = empty_heap();
        let vec = vec![5, 42, 1337, 77, -1, 0, 9999, 3, 65, 90, 1_000_000, -32];
        assert_push_equals_sorted_pop(&mut heap, vec);
        Ok(())
    })
}

#[test]
fn max_heap_with_multiple_levels() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut heap = empty_heap();
        let vec = vec![100, 10, 20, 30, 7, 8, 9, 17, 18, 29, 27, 28, 30];
        assert_push_equals_sorted_pop(&mut heap, vec);
        Ok(())
    })
}

/// A simple wrapper struct which is stored in the heap
/// for testing purposes (mostly to verify that custom
/// implemented `Ord` and `PartialOrd` are respected).
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Encode, Decode)]
struct V(u32);

#[test]
fn min_heap_with_multiple_levels() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut heap: BinaryHeap<V> = unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            BinaryHeap::allocate_using(&mut alloc).initialize_into(())
        };
        let vec = vec![
            V(100),
            V(10),
            V(20),
            V(30),
            V(7),
            V(8),
            V(9),
            V(17),
            V(18),
            V(29),
            V(27),
            V(28),
            V(30),
        ];
        assert_push_equals_sorted_pop(&mut heap, vec);
        Ok(())
    })
}

#[test]
fn extend() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut heap = filled_heap();
        let arr = [1, 2, 3];

        // when
        heap.extend(&arr);

        // then
        let mut expected = HashSet::new();
        expected.extend(heap.values());
        expected.extend(arr.iter());

        let actual = heap.values().collect::<HashSet<_>>();

        assert_eq!(actual, expected);

        Ok(())
    })
}
