#![cfg_attr(not(feature = "std"), no_std)]

pub use self::contract2::{
    Contract2,
    Contract2Ref,
};

#[ink::contract]
mod contract2 {
    use contract1::Contract1Ref;
    use ink::primitives::Key;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Contract2 {
        /// Stores a single `bool` value on the storage.
        value: u32,

        callee: Contract1Ref,
    }

    impl Contract2 {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(callee: Contract1Ref) -> Self {
            Self { value: 0, callee }
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> u32 {
            self.value
        }

        #[ink(message)]
        pub fn set_callee(&mut self, callee: Contract1Ref) {
            self.callee = callee;
        }

        #[ink(message)]
        pub fn get_callee(&self) -> AccountId {
            self.callee.get_address()
        }

        #[ink(message)]
        pub fn get_address(&self) -> AccountId {
            self.env().account_id()
        }

        #[ink(message)]
        pub fn get_key(&self) -> Key {
            <Self as ink::storage::traits::StorageKey>::KEY
        }

        #[ink(message, selector = _)]
        pub fn inc(&mut self) {
            ink::env::set_contract_storage(
                &<Self as ink::storage::traits::StorageKey>::KEY,
                self,
            );
            self.callee.inc().unwrap();
        }
    }
}
