//! A smart contract which demonstrates behavior of the `self.env().terminate()`
//! function. It terminates itself once `terminate_me()` is called.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod just_terminates {
    /// No storage is needed for this simple contract.
    #[ink(storage)]
    pub struct JustTerminate {}

    impl JustTerminate {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Terminates with the caller as beneficiary.
        #[ink(message)]
        pub fn terminate_me(&mut self) {
            self.env().terminate_contract(self.env().caller());
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn terminating_works() {
            // given
            let accounts =
                ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let contract_id = ink::env::test::callee::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
                contract_id,
                100,
            );
            let mut contract = JustTerminate::new();

            // when
            let should_terminate = move || contract.terminate_me().expect("TODO");

            // then
            ink::env::test::assert_contract_termination::<ink::env::DefaultEnvironment, _>(
                should_terminate,
                accounts.alice,
                100,
            );
        }
    }
}
