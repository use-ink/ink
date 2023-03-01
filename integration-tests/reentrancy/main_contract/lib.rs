#![cfg_attr(not(feature = "std"), no_std)]

pub use self::main_contract::{
    Error,
    MainContract,
    MainContractRef,
};

#[ink::contract]
mod main_contract {
    use core::mem::ManuallyDrop;
    use ink::env::{
        call::{
            build_call,
            Call,
        },
        CallFlags,
        DefaultEnvironment,
    };

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[derive(Default, Clone)]
    #[ink(storage)]
    pub struct MainContract {
        /// Stores a single `bool` value on the storage.
        value: u32,

        callee: Option<AccountId>,
    }

    #[derive(scale::Encode, scale::Decode, Debug, Ord, PartialOrd, Eq, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        CalleeReverted,
        Unknown,
    }

    impl MainContract {
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
            self.callee = Some(callee);
        }

        #[ink(message)]
        pub fn get_callee(&self) -> Option<AccountId> {
            self.callee
        }

        #[ink(message)]
        pub fn inc(&mut self) -> Result<u32, Error> {
            self.value += 1;

            if self.value > 1 {
                return Ok(self.value)
            }

            ink::env::set_contract_storage(
                &<Self as ink::storage::traits::StorageKey>::KEY,
                self,
            );

            build_call::<DefaultEnvironment>()
                .call_type(Call::new(self.callee.unwrap()))
                .call_flags(CallFlags::default().set_allow_reentry(true))
                .try_invoke()
                .unwrap_or_else(|err| panic!("failed to call callee: {:?}", err))
                .unwrap_or_else(|err| panic!("callee reverted: {:?}", err));

            let mut state = ink::env::get_contract_storage(
                &<Self as ink::storage::traits::StorageKey>::KEY,
            )
            .unwrap_or_else(|error| panic!("Failed to load contract state: {:?}", error))
            .unwrap_or_else(|| panic!("Contract state is not initialized"));

            core::mem::swap(self, &mut state);
            let _ = ManuallyDrop::new(state);

            Ok(self.value)
        }
    }
}
