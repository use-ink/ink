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

use super::Box as StorageBox;
use crate::{
    alloc,
    alloc::ContractPhase,
    traits::{
        KeyPtr,
        SpreadLayout,
    },
    Pack,
};
use core::{
    cmp::Ordering,
    convert::{
        AsMut,
        AsRef,
    },
    ops::{
        Deref,
        DerefMut,
    },
};
use ink_env::test::DefaultAccounts;
use ink_prelude::borrow::{
    Borrow,
    BorrowMut,
};
use ink_primitives::Key;

fn run_test<F>(f: F)
where
    F: FnOnce(DefaultAccounts<ink_env::DefaultEnvironment>),
{
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|default_accounts| {
        alloc::initialize(ContractPhase::Deploy);
        f(default_accounts);
        Ok(())
    })
    .unwrap()
}

#[test]
fn new_works() {
    run_test(|_| {
        let mut expected = 1;
        let mut boxed = StorageBox::new(expected);
        assert_eq!(StorageBox::get(&boxed), &expected);
        assert_eq!(StorageBox::get_mut(&mut boxed), &mut expected);
        assert_eq!(Deref::deref(&boxed), &expected);
        assert_eq!(DerefMut::deref_mut(&mut boxed), &mut expected);
        assert_eq!(AsRef::as_ref(&boxed), &expected);
        assert_eq!(AsMut::as_mut(&mut boxed), &mut expected);
        assert_eq!(Borrow::<i32>::borrow(&boxed), &expected);
        assert_eq!(BorrowMut::<i32>::borrow_mut(&mut boxed), &mut expected);
    })
}

#[test]
fn partial_eq_works() {
    run_test(|_| {
        let b1 = StorageBox::new(b'X');
        let b2 = StorageBox::new(b'Y');
        let b3 = StorageBox::new(b'X');
        assert!(<StorageBox<u8> as PartialEq>::ne(&b1, &b2));
        assert!(<StorageBox<u8> as PartialEq>::eq(&b1, &b3));
    })
}

#[test]
fn partial_ord_works() {
    run_test(|_| {
        let b1 = StorageBox::new(1);
        let b2 = StorageBox::new(2);
        let b3 = StorageBox::new(1);
        assert_eq!(
            <StorageBox<u8> as PartialOrd>::partial_cmp(&b1, &b2),
            Some(Ordering::Less)
        );
        assert_eq!(
            <StorageBox<u8> as PartialOrd>::partial_cmp(&b2, &b1),
            Some(Ordering::Greater)
        );
        assert_eq!(
            <StorageBox<u8> as PartialOrd>::partial_cmp(&b1, &b3),
            Some(Ordering::Equal)
        );
    })
}

#[test]
fn spread_layout_push_pull_works() {
    run_test(|_| {
        let b1 = StorageBox::new(b'A');
        assert_eq!(*b1, b'A');
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&b1, &mut KeyPtr::from(root_key));
        // Now load another instance of storage box from the same key and check
        // if both instances are equal:
        let b2 = SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(b1, b2);
        // We have to forget one of the storage boxes because we otherwise get
        // a double free panic since their `Drop` implementations both try to
        // free the same dynamic allocation.
        core::mem::forget(b2);
    })
}

#[test]
#[should_panic(expected = "storage entry was empty")]
fn spread_layout_clear_works() {
    run_test(|_| {
        let b1 = StorageBox::new(b'A');
        assert_eq!(*b1, b'A');
        let root_key = Key::from([0x42; 32]);
        // Manually clear the storage of `b1`. Then another load from the same
        // key region should panic since the entry is empty:
        SpreadLayout::push_spread(&b1, &mut KeyPtr::from(root_key));
        SpreadLayout::clear_spread(&b1, &mut KeyPtr::from(root_key));
        let b2: StorageBox<u8> = SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
        // We have to forget one of the storage boxes because we otherwise get
        // a double free panic since their `Drop` implementations both try to
        // free the same dynamic allocation.
        core::mem::forget(b2);
    })
}

#[test]
fn packed_layout_works() {
    run_test(|_| {
        let p1 = Pack::new((StorageBox::new(b'A'), StorageBox::new([0x01; 4])));
        assert_eq!(*p1.0, b'A');
        assert_eq!(*p1.1, [0x01; 4]);
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&p1, &mut KeyPtr::from(root_key));
        // Now load another instance of storage box from the same key and check
        // if both instances are equal:
        let p2: Pack<(StorageBox<u8>, StorageBox<[i32; 4]>)> =
            SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(p1, p2);
        // We have to forget one of the storage boxes because we otherwise get
        // a double free panic since their `Drop` implementations both try to
        // free the same dynamic allocation.
        core::mem::forget(p2);
    })
}

#[test]
fn recursive_pull_push_works() {
    run_test(|_| {
        let rec1 = StorageBox::new(StorageBox::new(b'A'));
        assert_eq!(**rec1, b'A');
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&rec1, &mut KeyPtr::from(root_key));
        // Now load another instance of storage box from the same key and check
        // if both instances are equal:
        let rec2: StorageBox<StorageBox<u8>> =
            SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(rec1, rec2);
        // We have to forget one of the storage boxes because we otherwise get
        // a double free panic since their `Drop` implementations both try to
        // free the same dynamic allocation.
        core::mem::forget(rec2);
    })
}

#[test]
#[should_panic(expected = "storage entry was empty")]
fn recursive_clear_works() {
    run_test(|_| {
        let rec1 = StorageBox::new(StorageBox::new(b'A'));
        assert_eq!(**rec1, b'A');
        let root_key = Key::from([0x42; 32]);
        // Manually clear the storage of `rec1`. Then another load from the same
        // key region should panic since the entry is empty:
        SpreadLayout::push_spread(&rec1, &mut KeyPtr::from(root_key));
        SpreadLayout::clear_spread(&rec1, &mut KeyPtr::from(root_key));
        let rec2: StorageBox<StorageBox<u8>> =
            SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
        // We have to forget one of the storage boxes because we otherwise get
        // a double free panic since their `Drop` implementations both try to
        // free the same dynamic allocation.
        core::mem::forget(rec2);
    })
}

#[test]
#[should_panic(expected = "encountered double free of dynamic storage: at index 0")]
fn double_free_panics() {
    run_test(|_| {
        let b1 = StorageBox::new(b'A');
        let root_key = Key::from([0x42; 32]);
        // Manually clear the storage of `rec1`. Then another load from the same
        // key region should panic since the entry is empty:
        SpreadLayout::push_spread(&b1, &mut KeyPtr::from(root_key));
        let b2: StorageBox<u8> = SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(b1, b2);
        // At this point both `b1` and `b2` are getting dropped trying to free
        // the same dynamic allocation which panics.
    })
}
