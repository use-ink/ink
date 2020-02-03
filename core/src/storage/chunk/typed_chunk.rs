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
    storage::alloc::{
        Allocate,
        AllocateUsing,
    },
};
use core::marker::PhantomData;
use ink_primitives::Key;

/// A chunk of typed cells.
///
/// Provides interpreted access with offset to the associated contract storage slot.
///
/// # Guarantees
///
/// - `Owned`
/// - `Typed`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug, PartialEq, Eq)]
pub struct TypedChunk<T> {
    /// The underlying key into the contract storage.
    key: Key,
    /// Marker to trick the Rust compiler into thinking that we actually make use of `T`.
    marker: PhantomData<fn() -> T>,
}

/// A single cell within a chunk of typed cells.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct TypedChunkCell<'a, T, M> {
    /// The underlying key that points to the typed cell.
    key: Key,
    /// Marker to trick the Rust compiler into thinking that we actually
    /// make use of `T` and `'a`.
    marker: PhantomData<fn() -> &'a (T, M)>,
}

/// Markers for a shared referenced typed chunk cell.
pub enum SharedTypedChunkCell {}
/// Markers for a exclusive referenced typed chunk cell.
pub enum ExclusiveTypedChunkCell {}

impl<'a, T> TypedChunkCell<'a, T, SharedTypedChunkCell> {
    /// Creates a new shared typed chunk cell from the given key.
    fn shared(key: Key) -> Self {
        Self {
            key,
            marker: Default::default(),
        }
    }
}

impl<'a, T> TypedChunkCell<'a, T, ExclusiveTypedChunkCell> {
    /// Creates a new exclusive typed chunk cell from the given key.
    fn exclusive(key: Key) -> Self {
        Self {
            key,
            marker: Default::default(),
        }
    }

    /// Removes the value stored in this cell.
    pub fn clear(self) {
        env::clear_contract_storage(self.key)
    }
}

impl<'a, T> TypedChunkCell<'a, T, ExclusiveTypedChunkCell>
where
    T: scale::Encode,
{
    /// Stores the value from the cell into the contract storage.
    pub fn store(self, new_value: &T) {
        env::set_contract_storage(self.key, new_value)
    }
}

impl<'a, T, M> TypedChunkCell<'a, T, M>
where
    T: scale::Decode,
{
    /// Loads the value from the storage into the cell.
    pub fn load(self) -> Option<T> {
        env::get_contract_storage(self.key)
            .map(|result| result.expect("could not decode T from storage chunk"))
    }
}

impl<T> AllocateUsing for TypedChunk<T> {
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            key: alloc.alloc(u32::max_value().into()),
            marker: Default::default(),
        }
    }
}

impl<T> scale::Encode for TypedChunk<T> {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.key.encode_to(dest)
    }
}

impl<T> scale::Decode for TypedChunk<T> {
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        Ok(Self {
            key: Key::decode(input)?,
            marker: Default::default(),
        })
    }
}

impl<T> TypedChunk<T> {
    /// Returns the underlying key.
    pub fn key(&self) -> Key {
        self.key
    }

    /// Returns a shared accessor the cell at the given index.
    fn cell_at(&self, index: u32) -> TypedChunkCell<T, SharedTypedChunkCell> {
        TypedChunkCell::shared(self.key + index)
    }

    /// Returns an exclusive accessor the cell at the given index.
    fn cell_at_mut(&mut self, index: u32) -> TypedChunkCell<T, ExclusiveTypedChunkCell> {
        TypedChunkCell::exclusive(self.key + index)
    }

    /// Removes the value stored in the `n`-th cell.
    pub fn clear(&mut self, index: u32) {
        self.cell_at_mut(index).clear()
    }
}

impl<T> TypedChunk<T>
where
    T: scale::Decode,
{
    /// Loads the value stored in the storage at the given index if any.
    ///
    /// # Panics
    ///
    /// If decoding of the loaded bytes fails.
    pub fn load(&self, index: u32) -> Option<T> {
        self.cell_at(index).load()
    }
}

impl<T> TypedChunk<T>
where
    T: scale::Encode,
{
    /// Stores the value into the cell at the given index.
    pub fn store(&mut self, index: u32, new_value: &T) {
        self.cell_at_mut(index).store(new_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        env,
        env::Result,
    };

    fn create_typed_chunk() -> TypedChunk<u32> {
        unsafe {
            let mut alloc =
                crate::storage::alloc::BumpAlloc::from_raw_parts(Key([0x0; 32]));
            TypedChunk::allocate_using(&mut alloc)
        }
    }

    #[test]
    fn simple() -> Result<()> {
        env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
            const TEST_LEN: u32 = 5;
            let mut chunk = create_typed_chunk();

            // Invariants after initialization
            for i in 0..TEST_LEN {
                assert_eq!(chunk.load(i), None);
            }

            // Store some elements
            for i in 0..TEST_LEN {
                chunk.store(i, &i);
                assert_eq!(chunk.load(i), Some(i));
            }

            // Clear all elements.
            for i in 0..TEST_LEN {
                chunk.clear(i);
                assert_eq!(chunk.load(i), None);
            }
            Ok(())
        })
    }

    /// Returns the current number of total contract storage reads and writes.
    fn get_contract_storage_rw() -> (usize, usize) {
        let contract_account_id = env::account_id::<env::DefaultEnvTypes>().unwrap();
        env::test::get_contract_storage_rw::<env::DefaultEnvTypes>(&contract_account_id)
            .unwrap()
    }

    #[test]
    fn count_reads_writes() -> Result<()> {
        env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
            const TEST_LEN: u32 = 5;
            let mut chunk = create_typed_chunk();

            // Reads and writes after init.
            assert_eq!(get_contract_storage_rw(), (0, 0),);

            // Loading from all cells.
            for i in 0..TEST_LEN {
                chunk.load(i);
                assert_eq!(get_contract_storage_rw(), (i as usize + 1, 0));
            }
            assert_eq!(get_contract_storage_rw(), (TEST_LEN as usize, 0));

            // Writing to all cells.
            for i in 0..TEST_LEN {
                chunk.store(i, &i);
                assert_eq!(
                    get_contract_storage_rw(),
                    (TEST_LEN as usize, i as usize + 1)
                );
            }
            assert_eq!(
                get_contract_storage_rw(),
                (TEST_LEN as usize, TEST_LEN as usize)
            );

            // Loading multiple times from a single cell.
            const LOAD_REPEATS: usize = 3;
            for n in 0..LOAD_REPEATS {
                chunk.load(0);
                assert_eq!(
                    get_contract_storage_rw(),
                    (TEST_LEN as usize + n + 1, TEST_LEN as usize,)
                );
            }
            assert_eq!(
                get_contract_storage_rw(),
                (TEST_LEN as usize + LOAD_REPEATS, TEST_LEN as usize,)
            );

            // Storing multiple times to a single cell.
            const STORE_REPEATS: usize = 3;
            for n in 0..STORE_REPEATS {
                chunk.store(0, &10);
                assert_eq!(
                    get_contract_storage_rw(),
                    (TEST_LEN as usize + LOAD_REPEATS, TEST_LEN as usize + n + 1,)
                );
            }
            assert_eq!(
                get_contract_storage_rw(),
                (
                    TEST_LEN as usize + LOAD_REPEATS,
                    TEST_LEN as usize + STORE_REPEATS,
                )
            );
            Ok(())
        })
    }
}
