//! A smart contract to test reading and writing contract storage

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract_storage {
    use ink::prelude::string::String;

    /// A contract for testing reading and writing contract storage.
    #[ink(storage)]
    #[derive(Default)]
    pub struct ContractStorage;

    impl ContractStorage {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Read from the contract storage slot, consuming all the data from the buffer.
        #[ink(message)]
        pub fn set_and_get_storage_same_size(&self) -> Result<(), String> {
            let key = 0u32;
            let value = [0x42; 32];
            ink::env::set_contract_storage(&key, &value);
            let loaded_value = ink::env::get_contract_storage(&key)
                .expect("no value stored under the key");
            assert_eq!(loaded_value, Some(value));
            Ok(())
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
