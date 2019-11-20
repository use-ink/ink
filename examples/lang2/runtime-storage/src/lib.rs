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

#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use scale::{
    Encode as _,
    KeyedVec as _,
};
use ink_core::{
    memory::format,
};
use ink_lang2 as ink;

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
    /// This simple contract reads a value from runtime storage
    #[ink(storage)]
    struct RuntimeStorage {
    }

    impl RuntimeStorage {
        #[ink(constructor)]
        fn new(&mut self) {}

        /// Returns the account balance, read directly from runtime storage
        #[ink(message)]
        fn get_balance(&self, account: AccountId) -> Balance {
            const BALANCE_OF: &[u8] = b"Balances FreeBalance";
            let key = crypto::blake2_256(&account.to_keyed_vec(BALANCE_OF));
            let result = self.env().get_runtime_storage::<Balance>(&key[..]);
            result.unwrap_or_default()
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
