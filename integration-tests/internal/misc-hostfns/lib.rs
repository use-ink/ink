#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
mod misc_hostfns {
    #[ink(storage)]
    pub struct MiscHostfns {}

    impl MiscHostfns {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Checks that the host functions `address`, `to_account_id`,
        /// and `account_id` work together.
        #[ink(message)]
        pub fn addr_account_id(&self) {
            let addr = self.env().address();
            let to_account_id = self.env().to_account_id(addr);
            let account_id = self.env().account_id();
            assert_eq!(
                to_account_id, account_id,
                "failed asserting equality for the account id"
            );
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn works() {
            let contract = MiscHostfns::new();
            contract.addr_account_id();
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_addr_account_id_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = MiscHostfnsRef::new();
            let contract = client
                .instantiate("misc_hostfns", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<MiscHostfns>();

            // then
            let acc = call_builder.addr_account_id();
            let _call_res = client
                .call(&ink_e2e::alice(), &acc)
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            Ok(())
        }
    }
}
