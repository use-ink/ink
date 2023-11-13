#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::contract_to_call::{
    ContractToCall,
    ContractToCallRef,
};

#[ink::contract]
mod contract_to_call {
    #[ink(storage)]
    pub struct ContractToCall {}

    impl ContractToCall {
        /// Creates a new Template contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Returns true if this contract has been called as the origin.
        #[ink(message)]
        pub fn im_the_origin(&self) -> bool {
            self.env().caller_is_origin()
        }
    }

    impl Default for ContractToCall {
        fn default() -> Self {
            Self::new()
        }
    }
}
