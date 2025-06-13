#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::contract1::Contract1Ref;

#[ink::contract]
mod contract1 {

    #[ink(storage)]
    pub struct Contract1 {
        x: u32,
    }

    impl Contract1 {
        /// Creates a new Template contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { x: 42 }
        }

        #[ink(message)]
        pub fn set_x(&mut self, x: u32) {
            self.x = x;
        }

        #[ink(message)]
        pub fn get_x(&self) -> u32 {
            self.x
        }

        /// Returns the address of the contract.
        #[ink(message)]
        pub fn own_addr(&self) -> Address {
            self.env().address()
        }

        /*
        // todo
        /// Returns the hash code of the contract through the function 'own_code_hash'.
        #[ink(message)]
        pub fn own_code_hash(&self) -> ink::H256 {
            self.env().own_code_hash().unwrap()
        }
         */
    }

    impl Default for Contract1 {
        fn default() -> Self {
            Self::new()
        }
    }
}
