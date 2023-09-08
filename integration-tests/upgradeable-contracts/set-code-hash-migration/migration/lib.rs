#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod incrementer {

    /// Storage struct matches exactly that of the original `incrementer` contract, from which
    /// we are migrating.
    #[ink(storage)]
    pub struct Incrementer {
        count: u32,
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

        /// Run the migration to the data layout for the upgraded contract.
        #[ink(message)]
        pub fn migrate(&self) {
        }

        /// Modifies the code which is used to execute calls to this contract address
        /// (`AccountId`).
        ///
        /// We use this to upgrade the contract logic. We don't do any authorization here,
        /// any caller can execute this method.
        ///
        /// In a production contract you would do some authorization here!
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: Hash) {
            self.env().set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!("Failed to `set_code_hash` to {code_hash:?} due to {err:?}")
            });
            ink::env::debug_println!("Switched code hash to {:?}.", code_hash);
        }
    }
}
