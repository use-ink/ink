// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use crate::Result;
use ink_primitives::Key;

#[test]
fn store_load_clear() -> Result<()> {
    crate::test::run_test::<crate::DefaultEnvironment, _>(|_| {
        let key = Key::from([0x42; 32]);
        let mut storage_entry = crate::storage_entry(&key);
        assert!(storage_entry.is_vacant());
        storage_entry.set(&[0x05_u8; 5]);
        assert!(storage_entry.is_occupied());
        assert_eq!(storage_entry.get::<[i8; 5]>(), Ok(Some([0x05; 5])),);
        storage_entry.clear();
        assert!(storage_entry.is_vacant());
        Ok(())
    })
}

#[test]
fn key_add() -> Result<()> {
    crate::test::run_test::<crate::DefaultEnvironment, _>(|_| {
        let key00 = Key::from([0x0; 32]);
        let key05 = key00 + 05_u64; // -> 5
        let key10 = key00 + 10_u64; // -> 10         | same as key55
        let key55 = key05 + 05_u64; // -> 5 + 5 = 10 | same as key10
        let mut storage_entry_55 = crate::storage_entry(&key55);
        storage_entry_55.set(&42);

        let mut storage_entry_10 = crate::storage_entry(&key10);
        assert_eq!(storage_entry_10.get::<i32>(), Ok(Some(42)));
        storage_entry_10.set(&1337);

        assert_eq!(storage_entry_55.get::<i32>(), Ok(Some(1337)));
        Ok(())
    })
}

#[test]
fn key_add_sub() -> Result<()> {
    crate::test::run_test::<crate::DefaultEnvironment, _>(|_| {
        let key0a = Key::from([0x0; 32]);
        let key1a = key0a + 1337_u64;
        let key2a = key0a + 42_u64;
        // let key3a = key0a + 52_u64;
        // let key2b = key3a - 10_u64;
        // let key1b = key2b - 42_u64;
        // let key0b = key1b + 2000_u64 - 663_u64; // same as key1a
        crate::storage_entry(&key0a).set(&1);
        crate::storage_entry(&key1a).set(&2);
        crate::storage_entry(&key2a).set(&3);
        // assert_eq!(crate::get_contract_storage::<i32>(&key2b), Some(Ok(3)));
        // assert_eq!(crate::get_contract_storage::<i32>(&key1b), Some(Ok(1)));
        // assert_eq!(crate::get_contract_storage::<i32>(&key0b), Some(Ok(2)));
        Ok(())
    })
}

#[test]
fn gas_price() -> crate::Result<()> {
    crate::test::run_test::<crate::DefaultEnvironment, _>(|_| {
        let gas_price = 2u32;
        crate::test::update_chain_spec(|chain_spec| {
            chain_spec.set_gas_price::<crate::DefaultEnvironment>(gas_price.into())
        })?;

        assert_eq!(
            2u128,
            crate::weight_to_fee::<crate::DefaultEnvironment>(1).unwrap()
        );
        assert_eq!(
            20u128,
            crate::weight_to_fee::<crate::DefaultEnvironment>(10).unwrap()
        );
        assert_eq!(
            6u128,
            crate::weight_to_fee::<crate::DefaultEnvironment>(3).unwrap()
        );

        Ok(())
    })
}
