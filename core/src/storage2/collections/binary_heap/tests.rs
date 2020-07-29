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

use super::BinaryHeap;
use crate::{
    env,
    storage2::traits::{
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    },
};
use ink_primitives::Key;

fn heap_from_slice<T>(slice: &[T]) -> BinaryHeap<T>
where
    T: Clone + PackedLayout + Ord,
{
    slice.iter().cloned().collect()
}

/// Creates a heap populated with `n` consecutive values.
fn heap_of_size(n: u32) -> BinaryHeap<u32> {
    std::iter::repeat(0u32)
        .take(n as usize)
        .enumerate()
        .map(|(i, _)| i as u32 + 1)
        .collect()
}

#[test]
fn new_binary_heap_works() {
    // `BinaryHeap::new`
    let mut heap = <BinaryHeap<i32>>::new();
    assert!(heap.is_empty());
    assert_eq!(heap.len(), 0);
    assert!(heap.iter().next().is_none());
    assert_eq!(heap.peek(), None);
    assert_eq!(heap.pop(), None);
    // `BinaryHeap::default`
    let mut default = <BinaryHeap<i32> as Default>::default();
    assert!(default.is_empty());
    assert_eq!(default.len(), 0);
    assert!(default.iter().next().is_none());
    assert_eq!(default.peek(), None);
    assert_eq!(default.pop(), None);
    // `BinaryHeap::new` and `BinaryHeap::default` should be equal.
    assert_eq!(heap, default);
}

#[test]
fn from_iterator_works() {
    let some_primes = [1, 2, 3, 5, 7, 11, 13];
    assert_eq!(some_primes.iter().copied().collect::<BinaryHeap<_>>(), {
        let mut vec = BinaryHeap::new();
        for prime in &some_primes {
            vec.push(*prime)
        }
        vec
    });
}

#[test]
fn from_empty_iterator_works() {
    assert_eq!(
        [].iter().copied().collect::<BinaryHeap<i32>>(),
        BinaryHeap::new(),
    );
}

#[test]
fn push_works() {
    let mut heap = heap_from_slice(&[2, 4, 9]);
    assert_eq!(heap.len(), 3);
    assert_eq!(*heap.peek().unwrap(), 9);
    heap.push(11);
    assert_eq!(heap.len(), 4);
    assert_eq!(*heap.peek().unwrap(), 11);
    heap.push(5);
    assert_eq!(heap.len(), 5);
    assert_eq!(*heap.peek().unwrap(), 11);
    heap.push(27);
    assert_eq!(heap.len(), 6);
    assert_eq!(*heap.peek().unwrap(), 27);
    heap.push(3);
    assert_eq!(heap.len(), 7);
    assert_eq!(*heap.peek().unwrap(), 27);
    heap.push(103);
    assert_eq!(heap.len(), 8);
    assert_eq!(*heap.peek().unwrap(), 103);
}

#[test]
fn peek_works() {
    let mut heap = <BinaryHeap<i32>>::new();
    heap.push(33);

    assert_eq!(heap.peek(), Some(&33));
}

#[test]
fn peek_and_pop_works() {
    let data = vec![2, 4, 6, 2, 1, 8, 10, 3, 5, 7, 0, 9, 1];
    let mut sorted = data.clone();
    sorted.sort();
    let mut heap = heap_from_slice(&data);
    while !heap.is_empty() {
        assert_eq!(heap.peek().unwrap(), sorted.last().unwrap());
        assert_eq!(heap.pop().unwrap(), sorted.pop().unwrap());
    }
}

// not sure we should have peek_mut, because it could violate the heap property?
// #[test]
// fn peek_mut_works() {
//     let mut heap = <BinaryHeap<i32>>::new();
//     heap.push(33);
//
//     let elem = heap.peek_mut().unwrap();
//     assert_eq!(heap.peek(), Some(&33));
// }

#[test]
fn spread_layout_push_pull_works() -> env::Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let heap1 = heap_from_slice(&[b'a', b'b', b'c', b'd']);
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&heap1, &mut KeyPtr::from(root_key));
        // Load the pushed binary heap into another instance and check that
        // both instances are equal:
        let heap2 =
            <BinaryHeap<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(heap1, heap2);
        Ok(())
    })
}

#[test]
#[should_panic(expected = "encountered empty storage cell")]
fn spread_layout_clear_works() {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let heap1 = heap_from_slice(&[b'a', b'b', b'c', b'd']);
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&heap1, &mut KeyPtr::from(root_key));
        // It has already been asserted that a valid instance can be pulled
        // from contract storage after a push to the same storage region.
        //
        // Now clear the associated storage from `heap1` and check whether
        // loading another instance from this storage will panic since the
        // heap's length property cannot read a value:
        SpreadLayout::clear_spread(&heap1, &mut KeyPtr::from(root_key));
        let _ =
            <BinaryHeap<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        Ok(())
    })
    .unwrap()
}

#[test]
fn clear_works_on_filled_heap() {
    let mut heap = heap_from_slice(&[b'a', b'b', b'c', b'd']);
    heap.clear();
    assert!(heap.is_empty());
}

#[test]
fn clear_works_on_empty_heap() {
    let mut heap = BinaryHeap::<u8>::default();
    heap.clear();
    assert!(heap.is_empty());
}

#[test]
fn push_largest_value_big_o_log_n() -> env::Result<()> {
    const CONST_READ_WRITES: usize = 2;

    for (n, log_n) in &[(2, 1usize), (4, 2), (8, 3), (16, 4), (32, 5), (64, 6)] {
        env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
            let heap1 = heap_of_size(*n);
            let root_key = Key::from([0x42; 32]);
            SpreadLayout::push_spread(&heap1, &mut KeyPtr::from(root_key));
            let contract_account =
                env::test::get_current_contract_account_id::<env::DefaultEnvTypes>()?;

            let mut lazy_heap = <BinaryHeap<u32> as SpreadLayout>::pull_spread(
                &mut KeyPtr::from(root_key),
            );

            let (base_reads, base_writes) = env::test::get_contract_storage_rw::<
                env::DefaultEnvTypes,
            >(&contract_account)?;
            assert_eq!((base_reads as u32, base_writes as u32), (0, n + 1));

            let largest_value = n + 1;
            lazy_heap.push(largest_value);

            // write back to storage so we can see how many writes required
            SpreadLayout::push_spread(&lazy_heap, &mut KeyPtr::from(root_key));

            let (reads, writes) = env::test::get_contract_storage_rw::<
                env::DefaultEnvTypes,
            >(&contract_account)?;
            let net_reads = reads - CONST_READ_WRITES - base_reads;
            let net_writes = writes - CONST_READ_WRITES - base_writes;

            assert_eq!(net_reads, *log_n, "Reads should be O(log n)");
            assert_eq!(net_writes, *log_n, "Writes should be O(log n)");
            // println!("READS: {} {}", net_reads, log_n);
            // println!("WRITES: {} {}", net_writes, log_n);
            Ok(())
        })?
    }
    Ok(())
}
