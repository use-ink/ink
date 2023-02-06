#![cfg_attr(not(feature = "std"), no_std)]

use ink::primitives::AccountId;
use sp_runtime::MultiAddress;

#[derive(scale::Encode)]
enum RuntimeCall {
    #[codec(index = 4)]
    Balances(BalancesCall),
}

#[derive(scale::Encode)]
enum BalancesCall {
    #[codec(index = 0)]
    Transfer {
        dest: MultiAddress<AccountId, ()>,
        #[codec(compact)]
        value: u128,
    },
}

#[ink::contract]
mod runtime_call {
    use crate::{
        BalancesCall,
        RuntimeCall,
    };

    #[ink(storage)]
    #[derive(Default)]
    pub struct RuntimeCaller;

    impl RuntimeCaller {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn make_transfer_through_runtime(
            &mut self,
            receiver: AccountId,
            value: Balance,
        ) {
            self.env()
                .call_runtime(&RuntimeCall::Balances(BalancesCall::Transfer {
                    dest: receiver.into(),
                    value,
                }))
                .expect("Should succeed");
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
            let mut contract = RuntimeCaller::new();
            contract.make_transfer_through_runtime(
                default_accounts::<DefaultEnvironment>().bob,
                10,
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

        const CONTRACT_BALANCE: Balance = 1_000_000_000_000_000;
        const TRANSFER_VALUE: Balance = 1_000_000_000;

        #[ink_e2e::test]
        #[ignore = "Requires that the pallet contract is configured with:\
            - `CallFilter` allowing for a transfer, e.g. `frame_support::traits::Everything`,\
            - `UnsafeUnstableInterface = ConstBool<true>`"]
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
                .balance(contract_acc_id)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_before = client
                .balance(receiver)
                .await
                .expect("Failed to get account balance");

            // when
            let transfer_message = build_message::<RuntimeCallerRef>(contract_acc_id)
                .call(|caller| {
                    caller.make_transfer_through_runtime(receiver, TRANSFER_VALUE)
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
