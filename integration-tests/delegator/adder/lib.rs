#![cfg_attr(not(feature = "std"), no_std)]

pub use self::adder::{Adder, AdderRef};

#[ink::contract]
mod adder {
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        CallFlags, DefaultEnvironment,
    };

    /// Increments the underlying `accumulator` value.
    #[ink(storage)]
    pub struct Adder {
        /// The `accumulator` contract.
        acc_contract: AccountId,
    }

    impl Adder {
        /// Creates a new `adder` from the given `accumulator`.
        #[ink(constructor)]
        pub fn new(acc_contract: AccountId) -> Self {
            Self { acc_contract }
        }

        /// Increases the `accumulator` value by some amount.
        #[ink(message)]
        pub fn inc(&mut self, by: i32) {
            // Debug message to check whether the contract gets called by the delegator
            let caller = self.env().caller();
            let message = ink::prelude::format!("got a call from {:?}", caller);
            ink::env::debug_println!("{}", &message);

            let _result = build_call::<DefaultEnvironment>()
                .call(self.acc_contract)
                .call_flags(CallFlags::default().set_tail_call(true))
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("inc")))
                        .push_arg(by),
                )
                .returns::<()>()
                .try_invoke();
            unreachable!("set_tail_call = true");
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use accumulator::AccumulatorRef;
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(additional_contracts = "../accumulator/Cargo.toml")]
        async fn accumulator(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate `accumulator` contract
            let init_value = 10;
            let acc_constructor = AccumulatorRef::new(init_value);
            let acc_contract_account_id = client
                .instantiate("accumulator", &ink_e2e::alice(), acc_constructor, 0, None)
                .await
                .expect("accumulator contract instantiation failed")
                .account_id;

            // Build `get` message of `accumulator` contract and execute
            let get_message =
                ink_e2e::build_message::<AccumulatorRef>(acc_contract_account_id.clone())
                    .call(|accumulator| accumulator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value);
            Ok(())
        }

        #[ink_e2e::test]
        async fn increase(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate `accumulator` contract
            let init_value = 10;
            let acc_constructor = AccumulatorRef::new(init_value);
            let acc_contract_account_id = client
                .instantiate("accumulator", &ink_e2e::alice(), acc_constructor, 0, None)
                .await
                .expect("accumulator contract instantiation failed")
                .account_id;

            // Build `get` message of `accumulator` contract and execute
            let get_message =
                ink_e2e::build_message::<AccumulatorRef>(acc_contract_account_id.clone())
                    .call(|accumulator| accumulator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value);

            // Instantiate `adder` contract
            let adder_constructor = AdderRef::new(acc_contract_account_id);
            let adder_contract_account_id = client
                .instantiate("adder", &ink_e2e::alice(), adder_constructor, 0, None)
                .await
                .expect("adder contract instantiation failed")
                .account_id;

            // Build `increase` message of `adder` contract and execute
            let increase = 10;
            let inc_message =
                ink_e2e::build_message::<AdderRef>(adder_contract_account_id.clone())
                    .call(|adder| adder.inc(increase));
            let inc_result = client.call(&ink_e2e::alice(), inc_message, 0, None).await;
            assert!(inc_result.is_ok());

            // Execute `get` message of `accumulator` contract
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value + increase);
            Ok(())
        }
    }
}
