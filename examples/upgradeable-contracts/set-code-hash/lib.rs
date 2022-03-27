#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod incrementer {

    #[ink(storage)]
    pub struct Incrementer {
        count: u32,
    }

    impl Incrementer {
        /// Creates a new counter smart contract initialized with the given base value.
        #[ink(constructor)]
        pub fn new(init_value: u32) -> Self {
            Self { count: init_value }
        }

        /// Creates a new counter smart contract initialized to `0`.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(0)
        }

        /// Splice `base` and `target` together.
        #[ink(message)]
        pub fn inc(&mut self) {
            self.count += 1;
            ink_env::debug_println!("count is {},use old code", self.count);
        }

        #[ink(message)]
        pub fn get(&self) -> u32 {
            self.count
        }

        /// Set new code to this contract.
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: [u8; 32]) {
            ink_env::set_code_hash(&code_hash).expect("Fail to set code.");
            ink_env::debug_println!("set code_hash success");
        }
    }
}
