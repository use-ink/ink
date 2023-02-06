#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod runtime_call {
    #[ink(storage)]
    pub struct RuntimeCaller;

    impl RuntimeCaller {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn make_transfer_through_runtime(
            &self,
            _value: Balance,
            _receiver: AccountId,
        ) {
            self.env().call_runtime(&()).expect("Should succeed");
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::default_accounts,
            DefaultEnvironment,
        };

        #[ink::test]
        #[should_panic(
            expected = "off-chain environment does not support `call runtime`"
        )]
        fn cannot_call_runtime_off_chain() {
            let contract = RuntimeCaller::new();
            contract.make_transfer_through_runtime(
                10,
                default_accounts::<DefaultEnvironment>().bob,
            );
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::{
            env::{
                test::default_accounts,
                DefaultEnvironment,
            },
            primitives::AccountId,
        };
        use ink_e2e::build_message;

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        const CONTRACT_BALANCE: Balance = 1_000_000;
        const TRANSFER_VALUE: Balance = 100;

        // requires call filter + unstable features set in runtime
        #[ink_e2e::test]
        async fn it_works(mut client: Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = RuntimeCallerRef::new();
            let contract_acc_id = client
                .instantiate(
                    "call-runtime",
                    &ink_e2e::alice(),
                    constructor,
                    CONTRACT_BALANCE,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            let receiver: AccountId = default_accounts::<DefaultEnvironment>().bob;

            let contract_balance_before = client
                .balance(contract_acc_id.clone())
                .await
                .expect("Failed to get account balance");
            let receiver_balance_before = client
                .balance(receiver.clone())
                .await
                .expect("Failed to get account balance");

            // when
            let transfer_message = build_message::<RuntimeCallerRef>(
                contract_acc_id.clone(),
            )
            .call(|caller| {
                caller.make_transfer_through_runtime(TRANSFER_VALUE, receiver.clone())
            });

            let _call_res = client
                .call(&ink_e2e::alice(), transfer_message, 0, None)
                .await
                .expect("call failed");

            // then
            let contract_balance_after = client
                .balance(contract_acc_id)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_after = client
                .balance(receiver)
                .await
                .expect("Failed to get account balance");

            assert_eq!(
                contract_balance_before,
                contract_balance_after + TRANSFER_VALUE
            );
            assert_eq!(
                receiver_balance_before + TRANSFER_VALUE,
                receiver_balance_after
            );

            Ok(())
        }
    }
}
