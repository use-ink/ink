//! A smart contract which demonstrates functionality of the
//! `pallet-revive` `System` precompile.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod system_precompile {
    use ink::U256;

    /// A contract for testing `Mapping` functionality.
    #[ink(storage)]
    #[derive(Default)]
    pub struct SystemPrecompile {}

    impl SystemPrecompile {
        /// New instance.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Returns the `minimum_balance`.
        #[ink(message)]
        pub fn minimum_balance(&mut self) -> U256 {
            self.env().minimum_balance()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn minimum_balance_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = SystemPrecompileRef::new();
            let contract = client
                .instantiate("system-precompile", &ink_e2e::eve(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<SystemPrecompile>();

            // when
            let min = client
                .call(&ink_e2e::eve(), &call_builder.minimum_balance())
                .dry_run()
                .await?
                .return_value();

            // then
            // both values are the default values used in the node configurations
            // and AssetHub
            let native_to_eth_ratio: U256 = 100_000_000.into();
            let existential_deposit: U256 = 100_000_000.into();
            let expected = native_to_eth_ratio.saturating_mul(existential_deposit);
            assert_eq!(min, expected);

            Ok(())
        }
    }
}
