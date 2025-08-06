#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::contract2::Contract2Ref;

#[ink::contract]
mod contract2 {

    #[ink(storage)]
    pub struct Contract2 {
        x: u64,
    }

    impl Contract2 {
        /// Creates a new Template contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { x: 0 }
        }

        #[ink(message)]
        pub fn get_x(&self) -> u32 {
            123456
        }

        #[ink(message)]
        pub fn set_x(&mut self, x: u64) {
            self.x = x;
        }

        /// Returns the address of the contract through the function 'own_address'.
        #[ink(message)]
        pub fn own_address(&self) -> Address {
            self.env().address()
        }
        /*
        /// Returns the hash code of the contract through the function 'own_code_hash'.
        #[ink(message)]
        pub fn own_code_hash(&self) -> Hash {
            self.env().address()
        }
         */
    }

    impl Default for Contract2 {
        fn default() -> Self {
            Self::new()
        }
    }
}
