//! A smart contract which demonstrates behavior of the `self.env().terminate()`
//! function. It terminates itself once `terminate_me()` is called.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
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
            self.env()
                .terminate_contract(self.env().caller())
                .expect("must succeed");
        }

        #[ink(message)]
        pub fn get(&self) -> u64 {
            13
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_contract_terminates<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = JustTerminateRef::new();
            let contract = client
                .instantiate("contract_terminate", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<JustTerminate>();

            // when
            let terminate_me = call_builder.terminate_me();
            let call_res = client
                .call(&ink_e2e::alice(), &terminate_me)
                .submit()
                .await
                .expect("terminate_me messages failed");

            // then
            assert!(call_res.contains_event("System", "KilledAccount"));
            assert!(call_res.contains_event("Balances", "Withdraw"));

            let get = call_builder.get();
            let call_res = client
                .call(&ink_e2e::alice(), &get)
                .submit()
                .await
                .expect("get message failed");
            assert!(call_res.dry_run.exec_result.result.unwrap().data.is_empty());

            Ok(())
        }
    }
}
