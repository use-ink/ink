#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod wildcard_selector {
    #[ink(storage)]
    pub struct WildcardSelector {
    }

    impl WildcardSelector {
        /// Creates a new wildcard selector smart contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { }
        }

        /// Wildcard selector handles messages with any selector.
        #[ink(message, selector = _)]
        pub fn wildcard(&mut self) {
            todo!()
        }

        /// Wildcard complement handles messages with a well-known reserved selector.
        #[ink(message, selector = @)]
        pub fn wildcard_complement(&mut self)  {
            todo!()
        }
    }

    // #[cfg(all(test, feature = "e2e-tests"))]
    // mod e2e_tests {
    //     use super::*;
    //     use ink_e2e::build_message;
    //
    //     type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    //
    //     #[ink_e2e::test]
    //     async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    //         // given
    //         let constructor = FlipperRef::new(false);
    //         let contract_acc_id = client
    //             .instantiate("flipper", &ink_e2e::alice(), constructor, 0, None)
    //             .await
    //             .expect("instantiate failed")
    //             .account_id;
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
