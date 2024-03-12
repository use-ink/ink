#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod flipper {
    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        /// Flips the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Returns the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }

        /// Returns the current balance of the Flipper.
        #[ink(message)]
        pub fn get_contract_balance(&self) -> Balance {
            self.env().balance()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{
            subxt::dynamic::Value,
            ChainBackend,
            ContractsBackend,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// Tests standard flipper scenario:
        /// - deploy the flipper contract with initial value `false`
        /// - flip the flipper
        /// - get the flipper's value
        /// - assert that the value is `true`
        #[ink_e2e::test(backend(runtime_only))]
        async fn it_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            const INITIAL_VALUE: bool = false;
            let mut constructor = FlipperRef::new(INITIAL_VALUE);

            let contract = client
                .instantiate(
                    "e2e-runtime-only-backend",
                    &ink_e2e::alice(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("deploy failed");

            // when
            let mut call_builder = contract.call_builder::<Flipper>();
            let _flip_res = client
                .call(&ink_e2e::bob(), &call_builder.flip())
                .submit()
                .await;

            // then
            let get_res = client
                .call(&ink_e2e::bob(), &call_builder.get())
                .dry_run()
                .await?;
            assert_eq!(get_res.return_value(), !INITIAL_VALUE);

            Ok(())
        }

        /// Tests runtime call scenario:
        /// - deploy the flipper contract
        /// - get the contract's balance
        /// - transfer some funds to the contract using runtime call
        /// - get the contract's balance again
        /// - assert that the contract's balance increased by the transferred amount
        #[ink_e2e::test(backend(runtime_only))]
        async fn runtime_call_works() -> E2EResult<()> {
            // given
            let mut constructor = FlipperRef::new(false);

            let contract = client
                .instantiate(
                    "e2e-runtime-only-backend",
                    &ink_e2e::alice(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("deploy failed");
            let call_builder = contract.call_builder::<Flipper>();

            let old_balance = client
                .call(&ink_e2e::alice(), &call_builder.get_contract_balance())
                .submit()
                .await
                .expect("get_contract_balance failed")
                .return_value();

            const ENDOWMENT: u128 = 10;

            // when
            let call_data = vec![
                Value::unnamed_variant("Id", [Value::from_bytes(contract.account_id)]),
                Value::u128(ENDOWMENT),
            ];
            client
                .runtime_call(
                    &ink_e2e::alice(),
                    "Balances",
                    "transfer_allow_death",
                    call_data,
                )
                .await
                .expect("runtime call failed");

            // then
            let new_balance = client
                .call(&ink_e2e::alice(), &call_builder.get_contract_balance())
                .submit()
                .await
                .expect("get_contract_balance failed")
                .return_value();

            assert_eq!(old_balance + ENDOWMENT, new_balance);
            Ok(())
        }

        /// Just instantiate a contract using non-default runtime.
        #[ink_e2e::test(backend(runtime_only(sandbox = ink_e2e::MinimalSandbox)))]
        async fn custom_runtime<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            client
                .instantiate(
                    "e2e-runtime-only-backend",
                    &ink_e2e::alice(),
                    &mut FlipperRef::new(false),
                )
                .submit()
                .await
                .expect("instantiate failed");

            Ok(())
        }
    }
}
