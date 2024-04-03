#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod e2e_call_runtime {
    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

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

        #[ink_e2e::test]
        async fn call_runtime_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = ContractRef::new();
            let contract = client
                .instantiate("e2e_call_runtime", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Contract>();

            let transfer_amount = 100_000_000_000u128;

            // when
            let call_data = vec![
                // A value representing a `MultiAddress<AccountId32, _>`. We want the
                // "Id" variant, and that will ultimately contain the
                // bytes for our destination address
                Value::unnamed_variant("Id", [Value::from_bytes(&contract.account_id)]),
                // A value representing the amount we'd like to transfer.
                Value::u128(transfer_amount),
            ];

            let get_balance = call_builder.get_contract_balance();
            let pre_balance = client
                .call(&ink_e2e::alice(), &get_balance)
                .dry_run()
                .await?
                .return_value();

            // Send funds from Alice to the contract using Balances::transfer
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
            let get_balance = call_builder.get_contract_balance();
            let get_balance_res = client
                .call(&ink_e2e::alice(), &get_balance)
                .dry_run()
                .await?;

            assert_eq!(
                get_balance_res.return_value(),
                pre_balance + transfer_amount
            );

            Ok(())
        }
    }
}
