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

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod runtime {
    use ink_core::{
        env,
        hash::{
            blake2_128_into,
            twox_128,
            twox_128_into,
        },
    };
    use ink_prelude::*;
    use scale::{Decode, Encode};

    /// All balance information for an account, mirroring the structure defined in the runtime.
    /// Copied from [substrate](https://github.com/paritytech/substrate/blob/2c87fe171bc341755a43a3b32d67560469f8daac/frame/system/src/lib.rs#L307)
    #[derive(Decode)]
    pub struct AccountData {
        free: Balance,
        _reserved: Balance,
        _misc_frozen: Balance,
        _fee_frozen: Balance,
    }

    /// Information of an account, mirroring the structure defined in the runtime
    /// Copied from [substrate](https://github.com/paritytech/substrate/blob/2c87fe171bc341755a43a3b32d67560469f8daac/frame/system/src/lib.rs#L307)
    #[derive(Decode)]
    pub struct AccountInfo {
        _nonce: u32,
        _refcount: u8,
        data: AccountData,
    }

    /// This simple contract reads a value from runtime storage
    #[ink(storage)]
    struct RuntimeStorage {}

    impl RuntimeStorage {
        #[ink(constructor)]
        fn new(&mut self) {}

        /// Returns an account's free balance, read directly from runtime storage
        #[ink(message)]
        fn get_balance(&self, account: AccountId) -> Balance {
            // build the key
            const MODULE_PREFIX: &[u8] = b"System";
            const STORAGE_PREFIX: &[u8] = b"Account";
            let mut buf = vec::Vec::new(); // todo: size?
            let mut key = twox_128(&MODULE_PREFIX, &mut buf).to_vec();
            twox_128_into(&STORAGE_PREFIX, &mut buf, &mut key);

            let encoded_accound = &account.encode();
            blake2_128_into(&encoded_accound, &mut buf, &mut key);
            key.extend_with_slice(&encoded_accound);

            // fetch from runtime storage
            let result = self.env().get_runtime_storage::<AccountInfo>(&key[..]);
            match result {
                Some(Ok(account_info)) => account_info.data.free,
                Some(Err(err)) => {
                    env::println(&format!("Error reading AccountInfo {:?}", err));
                    0
                },
                None => {
                    env::println(&format!("No data at key {:?}", key));
                    0
                }
            }
        }
    }

    #[cfg(all(test))]
    mod tests {
        use super::*;

        #[test]
        fn non_existent_account_returns_zero() {
            let contract = RuntimeStorage::new();
            let account: AccountId = [0u8; 32].into();
            assert_eq!(contract.get_balance(account), 0);
        }
    }
}
