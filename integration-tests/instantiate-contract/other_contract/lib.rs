#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::other_contract::OtherContractRef;

#[ink::contract()]
mod other_contract {

    #[ink(storage)]
    pub struct OtherContract {
        x: u32,
    }

    impl OtherContract {
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

        /// Returns the hash code of the contract through the function 'own_code_hash'.
        #[ink(message)]
        pub fn own_code_hash(&self) -> Hash {
            self.env().own_code_hash().unwrap()
        }
    }

    impl Default for OtherContract {
        fn default() -> Self {
            Self::new()
        }
    }
}
