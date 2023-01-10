#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

pub use self::contract1::{
    Contract1,
    Contract1Ref,
};

#[ink::contract]
mod contract1 {
    use ink::env::{
        call::{
            build_call,
            Call,
        },
        CallFlags,
        DefaultEnvironment,
    };
    use std::mem::ManuallyDrop;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[derive(Default)]
    #[ink(storage)]
    pub struct Contract1 {
        /// Stores a single `bool` value on the storage.
        value: u32,

        callee: AccountId,
    }

    impl Contract1 {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(message)]
        pub fn get(&self) -> u32 {
            self.value
        }

        #[ink(message)]
        pub fn get_address(&self) -> AccountId {
            self.env().account_id()
        }

        #[ink(message)]
        pub fn set_callee(&mut self, callee: AccountId) {
            self.callee = callee;
        }

        #[ink(message)]
        pub fn get_callee(&self) -> AccountId {
            self.callee
        }

        #[ink(message)]
        pub fn inc(&mut self) -> u32 {
            self.value = self.value + 1;
            println!("value {}", self.value);

            if self.value > 1 {
                return self.value
            }

            ink::env::set_contract_storage(
                &<Self as ink::storage::traits::StorageKey>::KEY,
                self,
            );

            build_call::<DefaultEnvironment>()
                .call_type(Call::new().callee(self.callee))
                .call_flags(CallFlags::default().set_allow_reentry(true))
                .fire()
                .unwrap_or_else(|err| panic!("failed to call callee: {:?}", err));

            let mut state = ink::env::get_contract_storage(
                &<Self as ink::storage::traits::StorageKey>::KEY,
            )
            .unwrap_or_else(|error| panic!("Failed to load contract state: {:?}", error))
            .unwrap_or_else(|| panic!("Contract state is not initialized"));
            core::mem::swap(self, &mut state);
            let _ = ManuallyDrop::new(state);

            self.value
        }
    }
}
