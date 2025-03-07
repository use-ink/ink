#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod wildcard_selector {
    use ink::prelude::string::String;

    #[ink(storage)]
    pub struct WildcardSelector {}

    struct MessageInput([u8; 4], String);
    impl ink::env::DecodeDispatch for MessageInput {
        fn decode_dispatch(input: &mut &[u8]) -> Result<Self, ink::env::DispatchError> {
            // todo improve code here
            let mut selector: [u8; 4] = [0u8; 4];
            selector.copy_from_slice(&input[..4]);
            let arg: String = ink::scale::Decode::decode(&mut &input[4..]).unwrap();
            Ok(MessageInput(selector, arg))
        }
    }

    impl WildcardSelector {
        /// Creates a new wildcard selector smart contract.
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {}
        }

        /// Wildcard selector handles messages with any selector.
        #[ink(message, selector = _)]
        pub fn wildcard(&mut self) {
            let MessageInput(_selector, _message) =
                ink::env::decode_input::<MessageInput>().unwrap();
            /*
            // todo
            ink::env::debug_println!(
                "Wildcard selector: {:?}, message: {}",
                _selector,
                _message
            );
            */
        }

        /// Wildcard complement handles messages with a well-known reserved selector.
        #[ink(message, selector = @)]
        pub fn wildcard_complement(&mut self, _message: String) {
            // todo
            // ink::env::debug_println!("Wildcard complement message: {}", _message);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        use ink::{
            env::call::utils::{
                Argument,
                ArgumentList,
                EmptyArgumentList,
            },
            primitives::reflect::ScaleEncoding,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
        type Environment = <WildcardSelectorRef as ink::env::ContractEnv>::Env;

        fn build_message(
            addr: ink::H160,
            selector: [u8; 4],
            message: String,
        ) -> ink_e2e::CallBuilderFinal<
            Environment,
            ArgumentList<
                Argument<String>,
                EmptyArgumentList<ScaleEncoding>,
                ScaleEncoding,
            >,
            (),
            ScaleEncoding,
        > {
            ink::env::call::build_call::<Environment>()
                .call(addr)
                .exec_input(
                    ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                        selector,
                    ))
                    .push_arg(message),
                )
                .returns::<()>()
        }

        #[ink_e2e::test]
        async fn arbitrary_selectors_handled_by_wildcard<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = WildcardSelectorRef::new();
            let contract_acc_id = client
                .instantiate("wildcard_selector", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed")
                .addr;

            // when
            const ARBITRARY_SELECTOR: [u8; 4] = [0xF9, 0xF9, 0xF9, 0xF9];
            let wildcard_message = "WILDCARD_MESSAGE 1".to_string();
            let wildcard = build_message(
                contract_acc_id,
                ARBITRARY_SELECTOR,
                wildcard_message.clone(),
            );

            let _result = client
                .call(&ink_e2e::bob(), &wildcard)
                .submit()
                .await
                .expect("wildcard failed");

            const ARBITRARY_SELECTOR_2: [u8; 4] = [0x01, 0x23, 0x45, 0x67];
            let wildcard_message2 = "WILDCARD_MESSAGE 2".to_string();
            let wildcard2 = build_message(
                contract_acc_id,
                ARBITRARY_SELECTOR_2,
                wildcard_message2.clone(),
            );

            let _result2 = client
                .call(&ink_e2e::bob(), &wildcard2)
                .submit()
                .await
                .expect("wildcard failed");

            // then
            /*
            // todo
            assert!(result.debug_message().contains(&format!(
                "Wildcard selector: {:?}, message: {}",
                ARBITRARY_SELECTOR, wildcard_message
            )));

            assert!(result2.debug_message().contains(&format!(
                "Wildcard selector: {:?}, message: {}",
                ARBITRARY_SELECTOR_2, wildcard_message2
            )));
            */

            Ok(())
        }

        #[ink_e2e::test]
        async fn wildcard_complement_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = WildcardSelectorRef::new();
            let contract_acc_id = client
                .instantiate("wildcard_selector", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed")
                .addr;

            // when
            let wildcard_complement_message = "WILDCARD COMPLEMENT MESSAGE".to_string();
            let wildcard = build_message(
                contract_acc_id,
                ink::IIP2_WILDCARD_COMPLEMENT_SELECTOR,
                wildcard_complement_message.clone(),
            );

            let _result = client
                .call(&ink_e2e::bob(), &wildcard)
                .submit()
                .await
                .expect("wildcard failed");

            // then
            /*
            // todo
            assert!(result.debug_message().contains(&format!(
                "Wildcard complement message: {}",
                wildcard_complement_message
            )));
            */

            Ok(())
        }
    }
}
