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

        /// Utilizes `ink_env::is_contract`.
        #[ink(message)]
        pub fn is_contract(&self) {
            let this_contract = self.env().address();
            assert!(self.env().is_contract(&this_contract));
            assert!(!self.env().is_contract(&self.env().caller()));

            const SYSTEM_PRECOMPILE: [u8; 20] =
                hex_literal::hex!("0000000000000000000000000000000000000900");
            assert!(
                self.env()
                    .is_contract(&ink::H160::from_slice(&SYSTEM_PRECOMPILE[..]))
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
            let contract = client
                .instantiate(
                    "misc_hostfns",
                    &ink_e2e::alice(),
                    &mut MiscHostfnsRef::new(),
                )
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<MiscHostfns>();

            // then
            let _call_res = client
                .call(&ink_e2e::alice(), &call_builder.addr_account_id())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_is_contract_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let contract = client
                .instantiate("misc_hostfns", &ink_e2e::bob(), &mut MiscHostfnsRef::new())
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<MiscHostfns>();

            // then
            let _call_res = client
                .call(&ink_e2e::bob(), &call_builder.is_contract())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            Ok(())
        }
    }
}
