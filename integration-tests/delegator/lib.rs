#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod delegator {
    use ink::env::Result as EnvResult;
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        CallFlags, DefaultEnvironment,
    };
    use ink::MessageResult;

    // Whether the contract calls the `adder` or `subber` contract.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
    pub enum Which {
        Adder,
        Subber,
    }

    #[ink(storage)]
    pub struct Delegator {
        /// Says which of `adder` or `subber` is currently in use.
        which: Which,
        /// The `accumulator` contract.
        acc_contract: AccountId,
        /// The `adder` contract.
        add_contract: AccountId,
        /// The `subber` contract.
        sub_contract: AccountId,
    }

    impl Delegator {
        #[ink(constructor)]
        pub fn new(
            acc_contract: AccountId,
            add_contract: AccountId,
            sub_contract: AccountId,
        ) -> Self {
            Delegator {
                which: Which::Adder,
                acc_contract,
                add_contract,
                sub_contract,
            }
        }

        #[ink(message)]
        pub fn get(&self) -> i32 {
            let result = build_call::<DefaultEnvironment>()
                .call(self.acc_contract)
                .gas_limit(0)
                .transferred_value(0)
                .call_flags(CallFlags::default())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "get"
                ))))
                .returns::<i32>()
                .try_invoke();

            match result {
                EnvResult::Ok(MessageResult::Ok(result)) => result,
                _ => unimplemented!(),
            }
        }

        #[ink(message)]
        pub fn change(&self, by: i32) {
            let (contract, method_selector) = match self.which {
                Which::Adder => (
                    self.add_contract,
                    Selector::new(ink::selector_bytes!("inc")),
                ),
                Which::Subber => (
                    self.sub_contract,
                    Selector::new(ink::selector_bytes!("dec")),
                ),
            };
            let _result = build_call::<DefaultEnvironment>()
                .call(contract)
                .call_flags(CallFlags::default().set_tail_call(true))
                .exec_input(ExecutionInput::new(method_selector).push_arg(by))
                .returns::<()>()
                .try_invoke();
            unreachable!("set_tail_call = true");
        }

        #[ink(message)]
        pub fn switch(&mut self) {
            match self.which {
                Which::Adder => {
                    self.which = Which::Subber;
                }
                Which::Subber => {
                    self.which = Which::Adder;
                }
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[ink::test]
        fn new() {
            let delegator = Delegator::new([0x06; 32].into(), [0x07; 32].into(), [0x08; 32].into());
            assert_eq!(delegator.acc_contract, [0x06; 32].into());
            assert_eq!(delegator.add_contract, [0x07; 32].into());
            assert_eq!(delegator.sub_contract, [0x08; 32].into());
            assert_eq!(delegator.which, Which::Adder);
        }

        #[ink::test]
        fn switch() {
            let mut delegator =
                Delegator::new([0x06; 32].into(), [0x07; 32].into(), [0x08; 32].into());
            assert_eq!(delegator.which, Which::Adder);
            delegator.switch();
            assert_eq!(delegator.which, Which::Subber);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use accumulator::AccumulatorRef;
        use adder::AdderRef;
        use subber::SubberRef;
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(
            additional_contracts = "accumulator/Cargo.toml adder/Cargo.toml subber/Cargo.toml"
        )]
        async fn instantiate_other(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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

            // Instantiate `subber` contract
            let subber_constructor = SubberRef::new(acc_contract_account_id);
            let _subber_contract_account_id = client
                .instantiate("subber", &ink_e2e::alice(), subber_constructor, 0, None)
                .await
                .expect("subber contract instantiation failed")
                .account_id;

            // Instantiate `adder` contract
            let adder_constructor = AdderRef::new(acc_contract_account_id);
            let _adder_contract_account_id = client
                .instantiate("adder", &ink_e2e::alice(), adder_constructor, 0, None)
                .await
                .expect("adder contract instantiation failed");
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
            let inc_message = ink_e2e::build_message::<AdderRef>(adder_contract_account_id.clone())
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

        #[ink_e2e::test]
        async fn decrease(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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

            // Instantiate `subber` contract
            let subber_constructor = SubberRef::new(acc_contract_account_id);
            let subber_contract_account_id = client
                .instantiate("subber", &ink_e2e::alice(), subber_constructor, 0, None)
                .await
                .expect("subber contract instantiation failed")
                .account_id;

            // Build `decrease` message of `subber` contract and execute
            let decrease = 10;
            let dec_message =
                ink_e2e::build_message::<SubberRef>(subber_contract_account_id.clone())
                    .call(|subber| subber.dec(decrease));
            let dec_result = client.call(&ink_e2e::alice(), dec_message, 0, None).await;
            assert!(dec_result.is_ok());

            // Execute `get` message of `accumulator` contract
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value - decrease);
            Ok(())
        }

        #[ink_e2e::test]
        async fn instantiate_delegator(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate `accumulator` contract
            let init_value = 10;
            let acc_constructor = AccumulatorRef::new(init_value);
            let acc_contract_account_id = client
                .instantiate("accumulator", &ink_e2e::alice(), acc_constructor, 0, None)
                .await
                .expect("accumulator contract instantiation failed")
                .account_id;

            // Instantiate `adder` contract
            let adder_constructor = AdderRef::new(acc_contract_account_id);
            let adder_contract_account_id = client
                .instantiate("adder", &ink_e2e::alice(), adder_constructor, 0, None)
                .await
                .expect("adder contract instantiation failed")
                .account_id;

            // Instantiate `subber` contract
            let subber_constructor = SubberRef::new(acc_contract_account_id);
            let subber_contract_account_id = client
                .instantiate("subber", &ink_e2e::alice(), subber_constructor, 0, None)
                .await
                .expect("subber contract instantiation failed")
                .account_id;

            // Instantiate `delegator` contract
            let del_constructor = DelegatorRef::new(
                acc_contract_account_id,
                adder_contract_account_id,
                subber_contract_account_id,
            );
            let _del_contract_account_id = client
                .instantiate("delegator", &ink_e2e::alice(), del_constructor, 0, None)
                .await
                .expect("delegator contract instantiation failed")
                .account_id;
            Ok(())
        }

        #[ink_e2e::test]
        async fn delegate(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate `accumulator` contract
            let init_value = 10;
            let acc_constructor = AccumulatorRef::new(init_value);
            let acc_contract_account_id = client
                .instantiate("accumulator", &ink_e2e::alice(), acc_constructor, 0, None)
                .await
                .expect("accumulator contract instantiation failed")
                .account_id;

            // Instantiate `adder` contract
            let adder_constructor = AdderRef::new(acc_contract_account_id);
            let adder_contract_account_id = client
                .instantiate("adder", &ink_e2e::alice(), adder_constructor, 0, None)
                .await
                .expect("adder contract instantiation failed")
                .account_id;

            // Instantiate `subber` contract
            let subber_constructor = SubberRef::new(acc_contract_account_id);
            let subber_contract_account_id = client
                .instantiate("subber", &ink_e2e::alice(), subber_constructor, 0, None)
                .await
                .expect("subber contract instantiation failed")
                .account_id;

            // Instantiate `delegator` contract
            let del_constructor = DelegatorRef::new(
                acc_contract_account_id,
                adder_contract_account_id,
                subber_contract_account_id,
            );
            let del_contract_account_id = client
                .instantiate("delegator", &ink_e2e::alice(), del_constructor, 0, None)
                .await
                .expect("delegator contract instantiation failed")
                .account_id;

            // Build `change` message of `delegator` contract and execute
            // (Add 10)
            let change = 10;
            let change_message =
                ink_e2e::build_message::<DelegatorRef>(del_contract_account_id.clone())
                    .call(|delegator| delegator.change(change));
            let change_result = client
                .call(&ink_e2e::alice(), change_message, 0, None)
                .await;
            assert!(change_result.is_ok());

            // Build `get` message of `delegator` contract and execute
            let get_message =
                ink_e2e::build_message::<DelegatorRef>(del_contract_account_id.clone())
                    .call(|delegator| delegator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value + change);

            // Build `switch` message of `delegator` contract and execute
            let switch_message =
                ink_e2e::build_message::<DelegatorRef>(del_contract_account_id.clone())
                    .call(|delegator| delegator.switch());
            let switch_result = client
                .call(&ink_e2e::alice(), switch_message, 0, None)
                .await;
            assert!(switch_result.is_ok());

            // Build `change` message of `delegator` contract and execute
            // (Substract 20)
            //
            // value = 20 (init_value + 10, from previous `change` message)
            let value = 20;
            let change = 20;
            let change_message =
                ink_e2e::build_message::<DelegatorRef>(del_contract_account_id.clone())
                    .call(|delegator| delegator.change(change));
            let change_result = client
                .call(&ink_e2e::alice(), change_message, 0, None)
                .await;
            assert!(change_result.is_ok());

            // Build `get` message of `delegator` contract and execute
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), value - change);

            // Build `get` message of `accumulator` contract and execute
            let get_message =
                ink_e2e::build_message::<AccumulatorRef>(acc_contract_account_id.clone())
                    .call(|accumulator| accumulator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), value - change);
            Ok(())
        }
    }
}
