#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod incrementer {

    /// Track a counter in storage.
    ///
    /// # Note
    ///
    /// We have changed the storage layout:
    /// - `count` is now a `u64` instead of a `u32`.
    /// - We have added a new field `inc_by` which controls how many to increment by.
    #[ink(storage)]
    pub struct Incrementer {
        count: u64,
        inc_by: u8,
    }

    impl Incrementer {
        /// Creates a new counter smart contract initialized with the given base value.
        ///
        /// # Note
        ///
        /// When upgrading using the `set_code_hash` workflow we only need to point to a
        /// contract's uploaded code hash, **not** an instantiated contract's
        /// `AccountId`.
        ///
        /// Because of this we will never actually call the constructor of this contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            unreachable!(
                "Constructors are not called when upgrading using `set_code_hash`."
            )
        }

        /// Increments the counter value which is stored in the contract's storage.
        ///
        /// # Note
        ///
        /// In this upgraded contract the value is incremented by the value in the
        /// `inc_by` field.
        #[ink(message)]
        pub fn inc(&mut self) {
            self.count = self.count.checked_add(self.inc_by.into()).unwrap();
        }

        /// Set the value by which the counter will be incremented.
        #[ink(message)]
        pub fn set_inc_by(&mut self, inc_by: u8) {
            self.inc_by = inc_by;
        }

        /// Returns the counter value which is stored in this contract's storage.
        #[ink(message)]
        pub fn get(&self) -> u64 {
            self.count
        }

        /// Modifies the code which is used to execute calls to this contract address
        /// (`AccountId`).
        ///
        /// We use this to upgrade the contract logic. We don't do any authorization here,
        /// any caller can execute this method.
        ///
        /// In a production contract you would do some authorization here!
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: ink::H256) {
            self.env().set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!("Failed to `set_code_hash` to {code_hash:?} due to {err:?}")
            });
            // todo
            // ink::env::debug_println!("Switched code hash to {:?}.", code_hash);
        }
    }
}
