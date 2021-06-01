// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

#[ink::trait_definition]
pub trait Erc1155 {
    #[ink(message)]
    fn safe_transfer_from(&mut self);

    #[ink(message)]
    fn safe_batch_transfer_from(&mut self);

    #[ink(message)]
    fn balance_of(&self);

    #[ink(message)]
    fn balance_of_batch(&self);

    #[ink(message)]
    fn set_approval_for_all(&mut self);

    #[ink(message)]
    fn is_approved_for_all(&self);
}

#[ink::trait_definition]
pub trait Erc1155TokenReceiver {
    #[ink(message)]
    fn on_erc_1155_received(&mut self);

    #[ink(message)]
    fn on_erc_1155_batch_received(&mut self);
}

#[ink::contract]
mod erc1155 {
    /// An ERC-1155 contract.
    #[ink(storage)]
    pub struct Erc1155 {}

    impl Erc1155 {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }
    }

    impl super::Erc1155 for Erc1155 {
        #[ink(message)]
        fn safe_transfer_from(&mut self) {
            todo!()
        }

        #[ink(message)]
        fn safe_batch_transfer_from(&mut self) {
            todo!()
        }

        #[ink(message)]
        fn balance_of(&self) {
            todo!()
        }

        #[ink(message)]
        fn balance_of_batch(&self) {
            todo!()
        }

        #[ink(message)]
        fn set_approval_for_all(&mut self) {
            todo!()
        }

        #[ink(message)]
        fn is_approved_for_all(&self) {
            todo!()
        }
    }

    impl super::Erc1155TokenReceiver for Erc1155 {
        #[ink(message)]
        fn on_erc_1155_received(&mut self) {
            todo!()
        }

        #[ink(message)]
        fn on_erc_1155_batch_received(&mut self) {
            todo!()
        }
    }

    /// Unit tests.
    #[cfg(not(feature = "ink-experimental-engine"))]
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn default_works() {
            assert!(true);
        }
    }
}
