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

use super::*;

use crate::{
	test_utils::run_test,
	storage::{self, Key},
};

/// Returns an empty storage vector at address `0x42`.
fn new_empty_vec<T>() -> storage::Vec<T> {
	use crate::storage::alloc::ForwardAlloc;
	unsafe {
		let mut fw_alloc = ForwardAlloc::from_raw_parts(Key([0x0; 32]));
		Vec::<T>::new_using_alloc(&mut fw_alloc)
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
	run_test(|| {
		let vec = new_empty_vec::<i32>();
		assert_eq!(vec.len(), 0);
		assert_eq!(vec.is_empty(), true);
		assert_eq!(vec.iter().next(), None);
	})
}

#[test]
fn simple() {
	run_test(|| {
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
	})
}

#[test]
fn pop_empty() {
	run_test(|| {
		let mut vec = new_empty_vec::<i32>();
		assert_eq!(vec.len(), 0);
		assert_eq!(vec.pop(), None);
		assert_eq!(vec.len(), 0);
	})
}

#[test]
fn iter() {
	run_test(|| {
		let vec = new_filled_vec();
		let mut iter = vec.iter();
		assert_eq!(iter.next(), Some(&5));
		assert_eq!(iter.next(), Some(&42));
		assert_eq!(iter.next(), Some(&1337));
		assert_eq!(iter.next(), Some(&77));
		assert_eq!(iter.next(), None);
	})
}

#[test]
fn iter_back() {
	run_test(|| {
		let vec = new_filled_vec();
		let mut iter = vec.iter();
		assert_eq!(iter.next_back(), Some(&77));
		assert_eq!(iter.next_back(), Some(&1337));
		assert_eq!(iter.next_back(), Some(&42));
		assert_eq!(iter.next_back(), Some(&5));
		assert_eq!(iter.next_back(), None);
	})
}

#[test]
fn get() {
	run_test(|| {
		let vec = new_filled_vec();
		assert_eq!(vec.get(0), Some(&5));
		assert_eq!(vec.get(1), Some(&42));
		assert_eq!(vec.get(2), Some(&1337));
		assert_eq!(vec.get(3), Some(&77));
		assert_eq!(vec.get(4), None);
		assert_eq!(vec.get(u32::max_value()), None);
	})
}

#[test]
fn index() {
	run_test(|| {
		let vec = new_filled_vec();
		assert_eq!(vec[0], 5);
		assert_eq!(vec[1], 42);
		assert_eq!(vec[2], 1337);
		assert_eq!(vec[3], 77);
	})
}

#[test]
fn index_mut() {
	run_test(|| {
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
	})
}

#[test]
fn index_comp() {
	run_test(|| {
		let vec = {
			let mut vec = new_empty_vec();
			vec.push(String::from("Hello"));
			vec.push(String::from(", "));
			vec.push(String::from("World!"));
			assert_eq!(vec.len(), 3);
			vec
		};
		assert_eq!(vec[0], "Hello");
	})
}

#[test]
#[should_panic]
fn index_fail_0() {
	run_test(|| {
		let vec = new_filled_vec();
		vec[4];
	})
}

#[test]
#[should_panic]
fn index_fail_1() {
	run_test(|| {
		let vec = new_filled_vec();
		vec[u32::max_value()];
	})
}

#[test]
#[should_panic]
fn index_fail_2() {
	run_test(|| {
		let vec = new_empty_vec::<i32>();
		vec[0];
	})
}

#[test]
fn mutate() {
	run_test(|| {
		let mut vec = new_filled_vec();
		assert_eq!(vec.mutate(0, |x| *x += 10), Some(&15));
		assert_eq!(vec.mutate(1, |x| *x *= 2), Some(&84));
		assert_eq!(vec.mutate(4, |x| *x *= 2), None);
		assert_eq!(vec.mutate(u32::max_value(), |_|()), None);
	})
}

#[test]
fn replace() {
	run_test(|| {
		let mut vec = new_filled_vec();
		assert_eq!(vec.replace(0, || 1), Some(5));
		assert_eq!(vec.get(0), Some(&1));
		assert_eq!(vec.replace(1, || 50), Some(42));
		assert_eq!(vec.get(1), Some(&50));
		assert_eq!(vec.replace(4, || 999), None);
		assert_eq!(vec.get(4), None);
	})
}

#[test]
fn swap() {
	run_test(|| {
		let mut vec = new_filled_vec();
		assert_eq!(vec.get(1), Some(&42));
		assert_eq!(vec.get(3), Some(&77));
		vec.swap(1, 3);
		assert_eq!(vec.get(1), Some(&77));
		assert_eq!(vec.get(3), Some(&42));
	})
}

#[test]
fn swap_same() {
	run_test(|| {
		let mut vec = new_filled_vec();
		assert_eq!(vec.get(2), Some(&1337));
		// Does basically nothing.
		vec.swap(2, 2);
		assert_eq!(vec.get(2), Some(&1337));
	})
}

#[test]
#[should_panic]
fn swap_invalid() {
	run_test(|| {
		let mut vec = new_filled_vec();
		vec.swap(0, u32::max_value());
	})
}

#[test]
fn swap_remove() {
	run_test(|| {
		let mut vec = new_filled_vec();
		assert_eq!(vec.get(1), Some(&42));
		assert_eq!(vec.get(3), Some(&77));
		assert_eq!(vec.len(), 4);
		assert_eq!(vec.swap_remove(1), Some(42));
		assert_eq!(vec.get(1), Some(&77));
		assert_eq!(vec.get(3), None);
		assert_eq!(vec.len(), 3);
	})
}

#[test]
fn swap_remove_empty() {
	run_test(|| {
		let mut vec = new_empty_vec::<i32>();
		assert_eq!(vec.swap_remove(0), None);
	})
}

#[test]
fn iter_size_hint() {
	run_test(|| {
		let vec = new_filled_vec();
		let mut iter = vec.iter();
		assert_eq!(iter.size_hint(), (4, Some(4)));
		iter.next();
		assert_eq!(iter.size_hint(), (3, Some(3)));
	})
}
