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

use super::BitStash;
use crate::traits::{
    KeyPtr,
    SpreadLayout,
};
use ink_primitives::Key;

cfg_if::cfg_if! {
    if #[cfg(miri)] {
        // We need to lower the test allocations because miri's stacked borrows
        // analysis currently is super linear for some work loads.
        // Read more here: https://github.com/rust-lang/miri/issues/1367
        const TEST_ALLOCATIONS: u32 = 10;
    } else {
        const TEST_ALLOCATIONS: u32 = 10_000;
    }
}

#[test]
fn default_works() {
    let default = BitStash::default();
    assert_eq!(default.get(0), None);
}

#[test]
fn put_and_take_works() {
    let mut default = BitStash::default();
    assert_eq!(default.get(0), None);
    assert_eq!(default.put(), 0);
    assert_eq!(default.get(0), Some(true));
    assert_eq!(default.take(0), Some(true));
    assert_eq!(default.get(0), Some(false));
}

#[test]
fn put_works() {
    let mut default = BitStash::default();
    for i in 0..TEST_ALLOCATIONS {
        assert_eq!(default.get(i), None);
        assert_eq!(default.put(), i);
        assert_eq!(default.get(i), Some(true));
    }
}

fn filled_bitstash() -> BitStash {
    let mut default = BitStash::default();
    for i in 0..TEST_ALLOCATIONS {
        assert_eq!(default.put(), i);
        assert_eq!(default.get(i), Some(true));
    }
    default
}

#[test]
fn get_works() {
    let mut default = filled_bitstash();
    // Remove all bits at indices `(% 3 == 0)` and `(% 5 == 0)`.
    for i in 0..TEST_ALLOCATIONS {
        if i % 3 == 0 || i % 5 == 0 {
            default.take(i);
        }
    }
    for i in 0..TEST_ALLOCATIONS {
        let expected = !(i % 3 == 0 || i % 5 == 0);
        assert_eq!(default.get(i), Some(expected));
    }
}

#[test]
fn take_in_order_works() {
    let mut default = filled_bitstash();
    for i in 0..TEST_ALLOCATIONS {
        assert_eq!(default.get(i), Some(true));
        assert_eq!(default.take(i), Some(true));
        assert_eq!(default.get(i), Some(false));
    }
}

#[test]
fn take_in_rev_order_works() {
    let mut default = filled_bitstash();
    for i in (0..TEST_ALLOCATIONS).rev() {
        assert_eq!(default.get(i), Some(true));
        assert_eq!(default.take(i), Some(true));
        assert_eq!(default.get(i), Some(false));
    }
}

#[test]
fn take_refill_works() {
    let mut default = filled_bitstash();
    for i in 0..TEST_ALLOCATIONS {
        assert_eq!(default.get(i), Some(true));
        assert_eq!(default.take(i), Some(true));
        assert_eq!(default.get(i), Some(false));
        assert_eq!(default.put(), i);
        assert_eq!(default.get(i), Some(true));
    }
}

#[test]
fn take_refill_rev_works() {
    let mut default = filled_bitstash();
    for i in (0..TEST_ALLOCATIONS).rev() {
        assert_eq!(default.get(i), Some(true));
        assert_eq!(default.take(i), Some(true));
        assert_eq!(default.get(i), Some(false));
        assert_eq!(default.put(), i);
        assert_eq!(default.get(i), Some(true));
    }
}

#[test]
fn spread_layout_push_pull_works() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let default = filled_bitstash();
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&default, &mut KeyPtr::from(root_key));
        let pulled = <BitStash as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(default, pulled);
        Ok(())
    })
    .unwrap()
}

#[test]
#[should_panic(expected = "encountered empty storage cell")]
fn spread_layout_clear_works() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let default = filled_bitstash();
        // First push the instance to the contract storage.
        // Then load a valid instance, check it and clear its associated storage.
        // Afterwards load the invalid instance from the same storage region
        // and try to interact with it which is expected to fail.
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&default, &mut KeyPtr::from(root_key));
        let pulled = <BitStash as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(default, pulled);
        SpreadLayout::clear_spread(&pulled, &mut KeyPtr::from(root_key));
        let invalid =
            <BitStash as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        // We have to prevent calling its destructor since that would also panic but
        // in an uncontrollable way.
        let mut invalid = core::mem::ManuallyDrop::new(invalid);
        // Now interact with invalid instance.
        let _ = invalid.put();
        Ok(())
    })
    .unwrap()
}
