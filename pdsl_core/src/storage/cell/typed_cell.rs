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
		NonCloneMarker,
		cell::RawCell,
		Allocator,
	},
};

/// A typed cell.
///
/// Provides interpreted access to the associated contract storage slot.
///
/// # Guarantees
///
/// - `Owned`
/// - `Typed`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TypedCell<T> {
	/// The associated raw cell.
	cell: RawCell,
	/// Marker that prevents this type from being `Copy` or `Clone` by accident.
	non_clone: NonCloneMarker<T>,
}

impl<T> parity_codec::Encode for TypedCell<T> {
	fn encode_to<W: parity_codec::Output>(&self, dest: &mut W) {
		self.cell.encode_to(dest)
	}
}

impl<T> parity_codec::Decode for TypedCell<T> {
	fn decode<I: parity_codec::Input>(input: &mut I) -> Option<Self> {
		RawCell::decode(input)
			.map(|raw_cell| Self{
				cell: raw_cell,
				non_clone: NonCloneMarker::default()
			})
	}
}

impl<T> TypedCell<T> {
	/// Allocates a new typed cell using the given storage allocator.
	///
	/// # Safety
	///
	/// The is unsafe because it does not check if the associated storage
	/// does not alias with storage allocated by other storage allocators.
	pub unsafe fn new_using_alloc<A>(alloc: &mut A) -> Self
	where
		A: Allocator
	{
		Self{
			cell: RawCell::new_using_alloc(alloc),
			non_clone: Default::default(),
		}
	}

	/// Removes the value stored in the cell.
	pub fn clear(&mut self) {
		self.cell.clear()
	}
}

impl<T> TypedCell<T>
where
	T: parity_codec::Decode
{
	/// Loads the value stored in the cell if any.
	pub fn load(&self) -> Option<T> {
		self.cell.load().and_then(|bytes| T::decode(&mut &bytes[..]))
	}
}

impl<T> TypedCell<T>
where
	T: parity_codec::Encode
{
	/// Stores the value into the cell.
	pub fn store(&mut self, val: &T) {
		self.cell.store(&T::encode(&val))
	}
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
	use super::*;
	use crate::storage::Key;

	use crate::{
		test_utils::run_test,
		env::TestEnv,
	};

	fn dummy_cell() -> TypedCell<i32> {
		unsafe {
			let mut alloc = crate::storage::alloc::BumpAlloc::from_raw_parts(
				Key([0x0; 32])
			);
			TypedCell::new_using_alloc(&mut alloc)
		}
	}

	#[test]
	fn simple() {
		run_test(|| {
			let mut cell = dummy_cell();
			assert_eq!(cell.load(), None);
			cell.store(&5);
			assert_eq!(cell.load(), Some(5));
			cell.clear();
			assert_eq!(cell.load(), None);
		})
	}

	#[test]
	fn count_reads() {
		run_test(|| {
			let cell = dummy_cell();
			assert_eq!(TestEnv::total_reads(), 0);
			cell.load();
			assert_eq!(TestEnv::total_reads(), 1);
			cell.load();
			cell.load();
			assert_eq!(TestEnv::total_reads(), 3);
		})
	}

	#[test]
	fn count_writes() {
		run_test(|| {
			let mut cell = dummy_cell();
			assert_eq!(TestEnv::total_writes(), 0);
			cell.store(&1);
			assert_eq!(TestEnv::total_writes(), 1);
			cell.store(&2);
			cell.store(&3);
			assert_eq!(TestEnv::total_writes(), 3);
		})
	}
}
