#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
pub mod incrementer {

    /// Track a counter in storage.
    ///
    /// # Note
    ///
    /// Is is important to realize that after the call to `set_code_hash` the contract's storage
    /// remains the same.
    ///
    /// If you change the storage layout in your storage struct you may introduce undefined
    /// behaviour to your contract!
    #[ink(storage)]
    #[derive(Default)]
    pub struct Incrementer {
        count: u32,
    }

    impl Incrementer {
        /// Creates a new counter smart contract initialized with the given base value.
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Increments the counter value which is stored in the contract's storage.
        #[ink(message)]
        pub fn inc(&mut self) {
            self.count += 1;
            ink::env::debug_println!(
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
            ink::env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!("Failed to `set_code_hash` to {code_hash:?} due to {err:?}")
            });
            ink::env::debug_println!("Switched code hash to {:?}.", code_hash);
        }
    }
}
