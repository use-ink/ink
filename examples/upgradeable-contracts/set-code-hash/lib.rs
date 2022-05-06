#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod incrementer {

    /// This struct contains the smart contract storage.
    /// The storage will always be retained, even when `set_code_hash` is called.
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

        /// Increments the counter value which is stored in the contract's storage.
        #[ink(message)]
        pub fn inc(&mut self) {
            self.count += 1;
            ink_env::debug_println!(
                "The new count is {}, it was modified using the original contract code.",
                self.count
            );
        }

        /// Returns the counter value which is stored in this contract's storage.
        #[ink(message)]
        pub fn get(&self) -> u32 {
            self.count
        }

        /// Modifies the code which is used to execute calls to this contract address (`AccountId`).
        ///
        /// We use this to upgrade the contract logic. We don't do any authorization here, any caller
        /// can execute this method. In a production contract you would do some authorization here.
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: [u8; 32]) {
            ink_env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!(
                    "Failed to `set_code_hash` to {:?} due to {:?}",
                    code_hash, err
                )
            });
            ink_env::debug_println!("Switched code hash to {:?}.", code_hash);
        }
    }
}
