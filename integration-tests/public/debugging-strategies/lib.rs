//! # Debugging Strategies
//!
//! This contract illustrates a number of strategies for debugging
//! contracts:
//!
//! * Emitting debugging events.
//! * The `pallet-revive` tracing API.
//! * Causing intentional reverts with a return value, in your contract.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod debugging_strategies {
    use ink::env::call::{
        ExecutionInput,
        Selector,
        build_call,
        build_create,
    };
    #[cfg(feature = "debug")]
    use ink::prelude::{
        borrow::ToOwned,
        format,
        string::String,
    };

    #[ink::event]
    #[cfg(feature = "debug")]
    pub struct DebugEvent {
        message: String,
    }

    /// Storage of the contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct DebuggingStrategies {
        value: bool,
    }

    impl DebuggingStrategies {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { value: true }
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            #[cfg(feature = "debug")]
            self.env().emit_event(DebugEvent {
                message: format!("received {:?}", self.env().transferred_value())
                    .to_owned(),
            });
            self.value
        }

        #[ink(message)]
        pub fn intentional_revert(&self) {
            #[cfg(feature = "debug")]
            ink::env::return_value(
                ink::env::ReturnFlags::REVERT,
                &format!("reverting with info: {:?}", self.env().transferred_value()),
            );
        }

        #[ink(message, payable)]
        pub fn instantiate_and_call(&mut self, code_hash: ink::H256) -> bool {
            let create_params = build_create::<DebuggingStrategiesRef>()
                .code_hash(code_hash)
                .endowment(0.into())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "new"
                ))))
                .returns::<DebuggingStrategiesRef>()
                .params();

            let other: DebuggingStrategiesRef = self.env()
                .instantiate_contract(&create_params)
                // todo
                // we do this to make sure the instantiated contract is always at a
                // different address
                // .salt(self.env().addr().to_vec())
                .unwrap_or_else(|error| {
                    panic!(
                        "Received an error from `pallet-revive` while instantiating: {error:?}"
                    )
                })
                .unwrap_or_else(|error| {
                    panic!("Received a `LangError` while instantiating: {error:?}")
                });

            let call = build_call()
                .call(ink::ToAddr::to_addr(&other))
                .transferred_value(0.into())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "get"
                ))))
                .returns::<bool>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {env_err:?}")
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {lang_err:?}"))
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::env::Environment;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// This test illustrates how to use debugging events.
        ///
        /// The contract is build with the `debug` feature enabled, thus
        /// we can have code in the contract that is utilized purely
        /// for testing, but not for release builds.
        #[ink_e2e::test(features = ["debug"])]
        async fn e2e_debugging_event_emitted<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = DebuggingStrategiesRef::new();
            let contract = client
                .instantiate("debugging_strategies", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<DebuggingStrategies>();

            // when
            let call_res = client
                .call(&ink_e2e::alice(), &call_builder.get())
                .submit()
                .await
                .expect("calling `get` message failed");

            // then
            // the contract will have emitted an event
            assert!(call_res.contains_event("Revive", "ContractEmitted"));
            let contract_events = call_res.contract_emitted_events()?;
            assert_eq!(1, contract_events.len());
            let contract_event = &contract_events[0];
            let debug_event: DebugEvent =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(debug_event.message, "received 0");

            Ok(())
        }

        /// This test illustrates how to decode a `Revive::ContractReverted`.
        #[ink_e2e::test(features = ["debug"])]
        async fn e2e_decode_intentional_revert<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = DebuggingStrategiesRef::new();
            let contract = client
                .instantiate("debugging_strategies", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<DebuggingStrategies>();

            // when
            let call_res = client
                .call(&ink_e2e::alice(), &call_builder.intentional_revert())
                .dry_run()
                .await
                .expect("calling `get` message failed");

            let return_data = call_res.return_data();
            assert!(call_res.did_revert());
            let revert_msg = String::from_utf8_lossy(return_data);
            assert!(revert_msg.contains("reverting with info: 0"));

            Ok(())
        }

        /// This test illustrates how to decode a `Revive::ContractReverted`.
        #[ink_e2e::test]
        async fn e2e_decode_revert<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = DebuggingStrategiesRef::new();
            let contract = client
                .instantiate("debugging_strategies", &ink_e2e::bob(), &mut constructor)
                .value(1_337_000_000)
                .dry_run()
                //.submit()
                .await
                .expect("instantiate failed");

            // when
            let return_data = contract.return_data();
            assert!(contract.did_revert());
            let revert_msg = String::from_utf8_lossy(return_data);
            assert!(revert_msg.contains("paid an unpayable message"));

            // todo show same for call
            let contract = client
                .instantiate("debugging_strategies", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<DebuggingStrategies>();

            // when
            let call_res = client
                .call(&ink_e2e::alice(), &call_builder.get())
                .value(1_337_000_000)
                .dry_run()
                .await
                .expect("calling `get` message failed");

            let return_data = call_res.return_data();
            assert!(call_res.did_revert());
            let revert_msg = String::from_utf8_lossy(return_data);
            assert!(
                revert_msg.contains(
                    "dispatching ink! message failed: paid an unpayable message"
                )
            );

            Ok(())
        }

        /// This test illustrates how to use the `pallet-revive` tracing functionality.
        #[ink_e2e::test]
        async fn e2e_tracing<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let mut constructor = DebuggingStrategiesRef::new();
            let contract = client
                .instantiate("debugging_strategies", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<DebuggingStrategies>();

            let call = call_builder.instantiate_and_call(contract.code_hash);
            let call_res = client
                .call(&ink_e2e::alice(), &call)
                .value(1_337_000_000)
                .submit()
                .await?;

            // when
            let trace: ink_e2e::CallTrace = call_res.trace.expect("trace must exist");
            assert_eq!(trace.calls.len(), 2);
            // This is how the object looks:
            // ```
            // CallTrace {
            //     from: 0x9621dde636de098b43efb0fa9b61facfe328f99d,
            //     gas: 1497105168000,
            //     gas_used: 1548337586000,
            //     to: 0xd71ff7085ed0e3e8b6c8e95eb6094f4311ae8e2f,
            //     input: Bytes(
            //         0x829da98747d85e35d0b3ca3c7ceeac09b63ec2754e6a05eb6d2d5b92fb916da126364dd4,
            //     ),
            //     output: Bytes(0x0001),
            //     error: None,
            //     revert_reason: None,
            //     calls: [
            //         CallTrace {
            //             from: 0xd71ff7085ed0e3e8b6c8e95eb6094f4311ae8e2f,
            //             gas: 711404887000,
            //             gas_used: 205987649000,
            //             to: 0xfd8bf44f34a2d2cec42b8ab31ede1bb1bc366e8e,
            //             input: Bytes(0x9bae9d5e),
            //             output: Bytes(0x0000),
            //             error: None,
            //             revert_reason: None,
            //             calls: [],
            //             logs: [],
            //             value: Some(0),
            //             call_type: Call,
            //         },
            //         CallTrace {
            //             from: 0xd71ff7085ed0e3e8b6c8e95eb6094f4311ae8e2f,
            //             gas: 124370129000,
            //             gas_used: 163567881000,
            //             to: 0xfd8bf44f34a2d2cec42b8ab31ede1bb1bc366e8e,
            //             input: Bytes(0x2f865bd9),
            //             output: Bytes(0x0001),
            //             error: None,
            //             revert_reason: None,
            //             calls: [],
            //             logs: [],
            //             value: Some(0),
            //             call_type: Call,
            //         },
            //     ],
            //     logs: [],
            //     value: Some(0),
            //     call_type: Call,
            // }
            // ```

            // then
            assert_eq!(
                trace.value,
                Some(ink::env::DefaultEnvironment::native_to_eth(1_337_000_000))
            );

            Ok(())
        }

        // todo add the same above, but for the sandbox
    }
}
