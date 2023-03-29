#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod wildcard_selector {
    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct WildcardSelector {
        wildcard_messages: Vec<([u8; 4], Vec<u8>)>,
        wildcard_complement_messages: Vec<Vec<u8>>,
    }

    impl WildcardSelector {
        /// Creates a new wildcard selector smart contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { wildcard_messages: Vec::new(), wildcard_complement_messages: Vec::new() }
        }

        /// Wildcard selector handles messages with any selector.
        #[ink(message, selector = _)]
        pub fn wildcard(&mut self) {
            let (selector, message) = ink::env::decode_input::<([u8; 4], Vec<u8>)>().unwrap();
            self.wildcard_messages.push((selector, message));
        }

        /// Wildcard complement handles messages with a well-known reserved selector.
        #[ink(message, selector = @)]
        pub fn wildcard_complement(&mut self, message: Vec<u8>) {
            self.wildcard_complement_messages.push(message);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{
            build_message,
            Message,
        };
        use scale::Encode as _;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = WildcardSelectorRef::new();
            let contract_acc_id = client
                .instantiate("wildcard_selector", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            const SELECTOR: [u8; 4] = [0xF9, 0xF9, 0xF9, 0xF9];
            let wildcard_message = "WILDCARD_MESSAGE".as_bytes().to_vec();

            let call_builder = ink::env::call::build_call::<ink::env::DefaultEnvironment>()
                .call(contract_acc_id.clone())
                .exec_input(
                    ink::env::call::ExecutionInput::new(
                        ink::env::call::Selector::new(SELECTOR)
                    )
                        .push_arg(wildcard_message.clone())
                )
                .returns::<()>();
            let exec_input = call_builder.params().exec_input().encode();
            let wildcard = Message::<ink::env::DefaultEnvironment, ()>::new(contract_acc_id.clone(), exec_input);

            let _ = client.call(&ink_e2e::bob(), wildcard, 0, None).await.expect("wildcard failed");

            Ok(())


            //
            //         let get = build_message::<FlipperRef>(contract_acc_id.clone())
            //             .call(|flipper| flipper.get());
            //         let get_res = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            //         assert!(matches!(get_res.return_value(), false));
            //
            //         // when
            //         let flip = build_message::<FlipperRef>(contract_acc_id.clone())
            //             .call(|flipper| flipper.flip());
            //         let _flip_res = client
            //             .call(&ink_e2e::bob(), flip, 0, None)
            //             .await
            //             .expect("flip failed");
            //
            //         // then
            //         let get = build_message::<FlipperRef>(contract_acc_id.clone())
            //             .call(|flipper| flipper.get());
            //         let get_res = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            //         assert!(matches!(get_res.return_value(), true));
            //
            //         Ok(())
            //     }
            //
            //     #[ink_e2e::test]
            //     async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            //         // given
            //         let constructor = FlipperRef::new_default();
            //
            //         // when
            //         let contract_acc_id = client
            //             .instantiate("flipper", &ink_e2e::bob(), constructor, 0, None)
            //             .await
            //             .expect("instantiate failed")
            //             .account_id;
            //
            //         // then
            //         let get = build_message::<FlipperRef>(contract_acc_id.clone())
            //             .call(|flipper| flipper.get());
            //         let get_res = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            //         assert!(matches!(get_res.return_value(), false));
            //
            //         Ok(())
            //     }
            // }
        }
    }
}
