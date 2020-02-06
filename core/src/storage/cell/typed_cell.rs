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
#[derive(Debug, PartialEq, Eq, Hash, scale::Encode, scale::Decode)]
pub struct TypedCell<T> {
    /// The associated storage key.
    key: Key,
    /// Marker to trick the Rust compiler.
    marker: PhantomData<fn() -> T>,
}

impl<T> AllocateUsing for TypedCell<T> {
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            key: alloc.alloc(1),
            marker: Default::default(),
        }
    }
}

impl<T> TypedCell<T> {
    /// Removes the value stored in the cell.
    pub fn clear(&mut self) {
        env::clear_contract_storage(self.key);
    }

    /// Returns the associated, internal raw key.
    pub fn key(&self) -> Key {
        self.key
    }
}

impl<T> TypedCell<T>
where
    T: scale::Decode,
{
    /// Loads the value stored in the cell if any.
    pub fn load(&self) -> Option<T> {
        env::get_contract_storage::<T>(self.key)
            .map(|result| result.expect("could not decode T from storage cell"))
    }
}

impl<T> TypedCell<T>
where
    T: scale::Encode,
{
    /// Stores the value into the cell.
    pub fn store(&mut self, new_value: &T) {
        env::set_contract_storage::<T>(self.key, &new_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        env::Result,
        storage::alloc::{
            AllocateUsing,
            BumpAlloc,
        },
    };

    fn dummy_cell() -> TypedCell<i32> {
        unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            TypedCell::allocate_using(&mut alloc)
        }
    }

    #[test]
    fn simple() -> Result<()> {
        env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
            let mut cell = dummy_cell();
            assert_eq!(cell.load(), None);
            cell.store(&5);
            assert_eq!(cell.load(), Some(5));
            cell.clear();
            assert_eq!(cell.load(), None);
            Ok(())
        })
    }

    #[test]
    fn count_reads() -> Result<()> {
        env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
            let cell = dummy_cell();
            let contract_account_id = env::account_id::<env::DefaultEnvTypes>()?;
            assert_eq!(
                env::test::get_contract_storage_rw::<env::DefaultEnvTypes>(
                    &contract_account_id
                )?,
                (0, 0)
            );
            cell.load();
            assert_eq!(
                env::test::get_contract_storage_rw::<env::DefaultEnvTypes>(
                    &contract_account_id
                )?,
                (1, 0)
            );
            cell.load();
            cell.load();
            assert_eq!(
                env::test::get_contract_storage_rw::<env::DefaultEnvTypes>(
                    &contract_account_id
                )?,
                (3, 0)
            );
            Ok(())
        })
    }

    #[test]
    fn count_writes() -> Result<()> {
        env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
            let mut cell = dummy_cell();
            let contract_account_id = env::account_id::<env::DefaultEnvTypes>()?;
            assert_eq!(
                env::test::get_contract_storage_rw::<env::DefaultEnvTypes>(
                    &contract_account_id
                )?,
                (0, 0)
            );
            cell.store(&1);
            assert_eq!(
                env::test::get_contract_storage_rw::<env::DefaultEnvTypes>(
                    &contract_account_id
                )?,
                (0, 1)
            );
            cell.store(&2);
            cell.store(&3);
            assert_eq!(
                env::test::get_contract_storage_rw::<env::DefaultEnvTypes>(
                    &contract_account_id
                )?,
                (0, 3)
            );
            Ok(())
        })
    }
}
