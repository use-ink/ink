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

mod crypto {
    /// Do a Blake2 256-bit hash and place result in `dest`.
    pub fn blake2_256_into(data: &[u8], dest: &mut [u8; 32]) {
        dest.copy_from_slice(blake2_rfc::blake2b::blake2b(32, &[], data).as_bytes());
    }

    /// Do a Blake2 256-bit hash and return result.
    pub fn blake2_256(data: &[u8]) -> [u8; 32] {
        let mut r = [0; 32];
        blake2_256_into(data, &mut r);
        r
    }
}

#[ink::contract(version = "0.1.0")]
mod runtime {
    use super::crypto;
    use scale::KeyedVec as _;

    /// This simple contract reads a value from runtime storage
    #[ink(storage)]
    struct RuntimeStorage {}

    impl RuntimeStorage {
        #[ink(constructor)]
        fn new(&mut self) {}

        /// Returns the account balance, read directly from runtime storage
        #[ink(message)]
        fn get_balance(&self, account: AccountId) -> Balance {
            const BALANCE_OF: &[u8] = b"Balances FreeBalance";
            let key = crypto::blake2_256(&account.to_keyed_vec(BALANCE_OF));
            let result = self.env().get_runtime_storage::<Balance>(&key[..]);
            result.unwrap_or_else(|| Ok(0)).unwrap_or_default()
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
