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
    env,
    memory::vec::Vec,
    storage::{
        alloc::{
            Allocate,
            AllocateUsing,
        },
        Key,
        NonCloneMarker,
    },
};

use parity_codec::{
    Decode,
    Encode,
};

/// A raw cell.
///
/// Provides uninterpreted and unformatted access to the associated contract storage slot.
///
/// # Guarantees
///
/// - `Owned`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct RawCell {
    /// The key to the associated constract storage slot.
    key: Key,
    /// Marker that prevents this type from being `Copy` or `Clone` by accident.
    non_clone: NonCloneMarker<()>,
}

impl AllocateUsing for RawCell {
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            key: alloc.alloc(1),
            non_clone: NonCloneMarker::default(),
        }
    }
}

impl RawCell {
    /// Loads the bytes stored in the cell if not empty.
    pub fn load(&self) -> Option<Vec<u8>> {
        unsafe { env::load(self.key) }
    }

    /// Stores the given bytes into the cell.
    pub fn store(&mut self, bytes: &[u8]) {
        unsafe { env::store(self.key, bytes) }
    }

    /// Removes the bytes stored in the cell.
    pub fn clear(&mut self) {
        unsafe { env::clear(self.key) }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;

    use crate::{
        storage::alloc::{
            AllocateUsing,
            BumpAlloc,
        },
        test_utils::run_test,
    };

    fn instantiate() -> RawCell {
        unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            RawCell::allocate_using(&mut alloc)
        }
    }

    #[test]
    fn simple() {
        run_test(|| {
            let mut cell = instantiate();
            assert_eq!(cell.load(), None);
            cell.store(b"Hello, World!");
            assert_eq!(cell.load(), Some(b"Hello, World!".to_vec()));
            cell.clear();
            assert_eq!(cell.load(), None);
        })
    }

    #[test]
    fn count_reads() {
        run_test(|| {
            let cell = instantiate();
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
            let mut cell = instantiate();
            assert_eq!(env::test::total_writes(), 0);
            cell.store(b"a");
            assert_eq!(env::test::total_writes(), 1);
            cell.store(b"b");
            cell.store(b"c");
            assert_eq!(env::test::total_writes(), 3);
        })
    }
}
