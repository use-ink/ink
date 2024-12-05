#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod incrementer {

    /// Storage struct matches exactly that of the original `incrementer` contract, from
    /// which we are migrating.
    #[ink(storage)]
    pub struct Incrementer {
        count: u32,
    }

    #[ink::storage_item]
    pub struct IncrementerNew {
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

        /// Run the migration to the data layout for the upgraded contract.
        /// Once the storage migration has successfully completed, the contract will be
        /// upgraded to the supplied code hash.
        ///
        /// In a production contract you would do some authorization here!
        ///
        /// # Note
        ///
        /// This function necessarily accepts a `&self` instead of a `&mut self` because
        /// we are modifying storage directly for the migration.
        ///
        /// The `self` in `&mut self` is the original `Incrementer` storage struct, and
        /// would be implicitly written to storage following the function execution,
        /// overwriting the migrated storage.
        #[ink(message)]
        pub fn migrate(&self, inc_by: u8, code_hash: ink::H256) {
            let incrementer_new = IncrementerNew {
                count: self.count as u64,
                inc_by,
            };

            // overwrite the original storage struct with the migrated storage struct,
            // which has a layout compatible with the new contract code.
            const STORAGE_KEY: u32 =
                <Incrementer as ink::storage::traits::StorageKey>::KEY;
            ink::env::set_contract_storage(&STORAGE_KEY, &incrementer_new);

            ink::env::set_code_hash::<<Self as ink::env::ContractEnv>::Env>(&code_hash)
                .unwrap_or_else(|err| {
                    panic!("Failed to `set_code_hash` to {code_hash:?} due to {err:?}")
                })
        }
    }
}
