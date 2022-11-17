#![cfg_attr(not(feature = "std"), no_std)]

pub use self::integration_flipper::{
    Flipper,
    FlipperRef,
};

#[ink::contract]
pub mod integration_flipper {
    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        /// Creates a new integration_flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Creates a new integration_flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// Flips the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Returns the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }

        /// Flips the current value of the Flipper's boolean.
        ///
        /// We should see the state being reverted here, no write should occur.
        #[ink(message)]
        #[allow(clippy::result_unit_err)]
        pub fn err_flip(&mut self) -> Result<(), ()> {
            self.flip();
            Err(())
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_can_flip_correctly(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = integration_flipper::constructors::default();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate `integration_flipper` failed")
                .account_id;

            let get_call_result = client
                .call(
                    &mut ink_e2e::alice(),
                    contract_acc_id.clone(),
                    integration_flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `get` failed");
            let initial_value = get_call_result
                .value
                .expect("Input is valid, call must not fail.");

            let flip_call_result = client
                .call(
                    &mut ink_e2e::alice(),
                    contract_acc_id.clone(),
                    integration_flipper::messages::flip(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flip` failed");
            assert!(
                flip_call_result.value.is_ok(),
                "Messages now return a `Result`, which should be `Ok` here."
            );

            let get_call_result = client
                .call(
                    &mut ink_e2e::alice(),
                    contract_acc_id.clone(),
                    integration_flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `get` failed");
            let flipped_value = get_call_result
                .value
                .expect("Input is valid, call must not fail.");
            assert!(flipped_value != initial_value);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_message_error_reverts_state(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = integration_flipper::constructors::default();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    contract_acc_id.clone(),
                    integration_flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `get` failed");
            let initial_value = get_call_result
                .value
                .expect("Input is valid, call must not fail.");

            let err_flip_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    contract_acc_id.clone(),
                    integration_flipper::messages::err_flip(),
                    0,
                    None,
                )
                .await;

            assert!(matches!(
                err_flip_call_result,
                Err(ink_e2e::Error::CallExtrinsic(_))
            ));

            let get_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    contract_acc_id.clone(),
                    integration_flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `get` failed");
            let flipped_value = get_call_result
                .value
                .expect("Input is valid, call must not fail.");
            assert!(flipped_value == initial_value);

            Ok(())
        }
    }
}
