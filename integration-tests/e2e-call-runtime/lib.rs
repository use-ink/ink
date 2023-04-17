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
            build_message,
            subxt::dynamic::Value,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn call_runtime_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = ContractRef::new();
            let contract_acc_id = client
                .instantiate("e2e_call_runtime", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let call_data = vec![
                // A value representing a `MultiAddress<AccountId32, _>`. We want the
                // "Id" variant, and that will ultimately contain the
                // bytes for our destination address
                Value::unnamed_variant("Id", [Value::from_bytes(&contract_acc_id)]),
                // A value representing the amount we'd like to transfer.
                Value::u128(100_000_000_000u128),
            ];

            // Send funds from Alice to the contract using Balances::transfer
            client
                .runtime_call(&ink_e2e::alice(), "Balances", "transfer", call_data)
                .await
                .expect("runtime call failed");

            // then
            let get_balance = build_message::<ContractRef>(contract_acc_id.clone())
                .call(|contract| contract.get_contract_balance());
            let get_balance_res = client
                .call_dry_run(&ink_e2e::alice(), &get_balance, 0, None)
                .await;

            assert!(matches!(
                get_balance_res.return_value(),
                100_000_000_000u128
            ));

            Ok(())
        }
    }
}
