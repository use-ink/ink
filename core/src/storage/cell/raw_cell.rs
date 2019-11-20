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

use scale::{
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
    /// The key to the associated contract storage slot.
    key: Key,
    /// Marker that prevents this type from being `Copy` or `Clone` by accident.
    non_clone: NonCloneMarker<()>,
}

impl AllocateUsing for RawCell {
    #[inline]
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

    /// Returns the associated, internal raw key.
    pub fn raw_key(&self) -> Key {
        self.key
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
