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
            Blake2x128,
        },
    };
    use ink_prelude::*;
    use scale::{
        Decode,
        Encode,
    };

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
        ///
        /// # Key Scheme
        ///
        /// A key for the [substrate storage map](https://github.com/paritytech/substrate/blob/dd97b1478b31a4715df7e88a5ebc6664425fb6c6/frame/support/src/storage/generator/map.rs#L28)
        /// is constructed with:
        ///
        /// ```nocompile
        /// Twox128(module_prefix) ++ Twox128(storage_prefix) ++ Hasher(encode(key))
        /// ```
        ///
        /// For the `System` module's `Account` map, the [hasher implementation](https://github.com/paritytech/substrate/blob/2c87fe171bc341755a43a3b32d67560469f8daac/frame/system/src/lib.rs#L349)
        /// is `blake2_128_concat`.
        #[ink(message)]
        fn get_balance(&self, account: AccountId) -> Balance {
            let key: [u8; 80] = [
                // Precomputed: Twox128("System")
                38, 170, 57, 78, 234, 86, 48, 224, 124, 72, 174, 12, 149, 88, 206, 247,
                // Precomputed: Twox128("Account")
                185, 157, 136, 14, 198, 129, 121, 156, 12, 243, 14, 136, 134, 55, 29, 169,
                // Space for `Blake128(account)`
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                // Space for `account` (assumes size [u8; 32])
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];

            let encoded_account = &account.encode();

            let mut output = [0x00_u8; 16];
            let mut accumulator = vec::Vec::with_capacity(16);

            let mut blake_128 = Blake2x128::from(&mut accumulator);
            blake_128.hash_raw_using(&encoded_account, &mut output);
            key.extend_from_slice(&output);
            key.extend_from_slice(&encoded_account);

            // fetch from runtime storage
            let result = self.env().get_runtime_storage::<AccountInfo>(&accumulator[..]);
            match result {
                Some(Ok(account_info)) => account_info.data.free,
                Some(Err(err)) => {
                    env::println(&format!("Error reading AccountInfo {:?}", err));
                    0
                }
                None => {
                    env::println(&format!("No data at key {:?}", accumulator));
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
