// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::storage::{
    alloc::{
        Allocate,
        AllocateUsing,
    },
    cell::RawCell,
    Key,
    NonCloneMarker,
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

impl<T> scale::Encode for TypedCell<T> {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.cell.encode_to(dest)
    }
}

impl<T> scale::Decode for TypedCell<T> {
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        RawCell::decode(input).map(|raw_cell| {
            Self {
                cell: raw_cell,
                non_clone: NonCloneMarker::default(),
            }
        })
    }
}

impl<T> AllocateUsing for TypedCell<T> {
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            cell: RawCell::allocate_using(alloc),
            non_clone: Default::default(),
        }
    }
}

impl<T> TypedCell<T> {
    /// Removes the value stored in the cell.
    pub fn clear(&mut self) {
        self.cell.clear()
    }

    /// Returns the associated, internal raw key.
    pub fn raw_key(&self) -> Key {
        self.cell.raw_key()
    }
}

impl<T> TypedCell<T>
where
    T: scale::Decode,
{
    /// Loads the value stored in the cell if any.
    pub fn load(&self) -> Option<T> {
        self.cell.load().map(|bytes| {
            T::decode(&mut &bytes[..]).expect(
                "[ink_core::TypedCell::load] Error: \
                 failed upon decoding",
            )
        })
    }
}

impl<T> TypedCell<T>
where
    T: scale::Encode,
{
    /// Stores the value into the cell.
    pub fn store(&mut self, val: &T) {
        self.cell.store(&T::encode(&val))
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use crate::{
        env,
        storage::Key,
    };

    use crate::{
        storage::alloc::{
            AllocateUsing,
            BumpAlloc,
        },
        test_utils::run_test,
    };

    fn dummy_cell() -> TypedCell<i32> {
        unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            TypedCell::allocate_using(&mut alloc)
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
            assert_eq!(env::test::total_reads(), 0);
            cell.load();
            assert_eq!(env::test::total_reads(), 1);
            cell.load();
            cell.load();
            assert_eq!(env::test::total_reads(), 3);
        })
    }

    #[test]
    fn count_writes() {
        run_test(|| {
            let mut cell = dummy_cell();
            assert_eq!(env::test::total_writes(), 0);
            cell.store(&1);
            assert_eq!(env::test::total_writes(), 1);
            cell.store(&2);
            cell.store(&3);
            assert_eq!(env::test::total_writes(), 3);
        })
    }
}
