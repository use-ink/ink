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

use crate::{
    env,
    env::Result,
};
use ink_primitives::Key;

#[test]
fn store_load_clear() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let key = Key::from([0x42; 32]);
        assert_eq!(env::get_contract_storage::<()>(&key), Ok(None));
        env::set_contract_storage(&key, &[0x05_u8; 5]);
        assert_eq!(
            env::get_contract_storage::<[i8; 5]>(&key),
            Ok(Some([0x05; 5])),
        );
        env::clear_contract_storage(&key);
        assert_eq!(env::get_contract_storage::<[u8; 5]>(&key), Ok(None));
        Ok(())
    })
}

#[test]
fn key_add() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let key00 = Key::from([0x0; 32]);
        let key05 = key00 + 05_u64; // -> 5
        let key10 = key00 + 10_u64; // -> 10         | same as key55
        let key55 = key05 + 05_u64; // -> 5 + 5 = 10 | same as key10
        env::set_contract_storage(&key55, &42);
        assert_eq!(env::get_contract_storage::<i32>(&key10), Ok(Some(42)));
        env::set_contract_storage(&key10, &1337);
        assert_eq!(env::get_contract_storage::<i32>(&key55), Ok(Some(1337)));
        Ok(())
    })
}

#[test]
fn key_add_sub() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let key0a = Key::from([0x0; 32]);
        let key1a = key0a + 1337_u64;
        let key2a = key0a + 42_u64;
        // let key3a = key0a + 52_u64;
        // let key2b = key3a - 10_u64;
        // let key1b = key2b - 42_u64;
        // let key0b = key1b + 2000_u64 - 663_u64; // same as key1a
        env::set_contract_storage(&key0a, &1);
        env::set_contract_storage(&key1a, &2);
        env::set_contract_storage(&key2a, &3);
        // assert_eq!(env::get_contract_storage::<i32>(&key2b), Some(Ok(3)));
        // assert_eq!(env::get_contract_storage::<i32>(&key1b), Some(Ok(1)));
        // assert_eq!(env::get_contract_storage::<i32>(&key0b), Some(Ok(2)));
        Ok(())
    })
}

#[test]
fn gas_price() -> env::Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let gas_price= 2u32;
        env::test::update_chain_spec(|chain_spec| {
            chain_spec.set_gas_price::<env::DefaultEnvTypes>(gas_price.into())
        })?;

        assert_eq!(2u128, env::weight_to_fee::<env::DefaultEnvTypes>(1).unwrap());
        assert_eq!(20u128, env::weight_to_fee::<env::DefaultEnvTypes>(10).unwrap());
        assert_eq!(6u128, env::weight_to_fee::<env::DefaultEnvTypes>(3).unwrap());

        Ok(())
    })
}
