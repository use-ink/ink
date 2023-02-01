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

    #[derive(scale::Encode, scale::Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct FlipperError;

    impl Flipper {
        /// Creates a new integration_flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Creates a new integration_flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        /// Attemps to create a new integration_flipper smart contract initialized with the given
        /// value.
        #[ink(constructor)]
        pub fn try_new(succeed: bool) -> Result<Self, FlipperError> {
            if succeed {
                Ok(Self::new(true))
            } else {
                Err(FlipperError)
            }
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
        use super::FlipperRef;
        use ink_e2e::build_message;
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_can_flip_correctly(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = FlipperRef::new_default();
            let contract_acc_id = client
                .instantiate(
                    "integration_flipper",
                    &ink_e2e::alice(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("Instantiate `integration_flipper` failed")
                .account_id;

            let get = build_message::<FlipperRef>(contract_acc_id.clone())
                .call(|contract| contract.get());
            let initial_value = client
                .call_dry_run(&ink_e2e::alice(), &get, 0, None)
                .await
                .return_value();

            let flip = build_message::<FlipperRef>(contract_acc_id)
                .call(|contract| contract.flip());
            let flip_call_result = client
                .call(&ink_e2e::alice(), flip, 0, None)
                .await
                .expect("Calling `flip` failed");
            assert!(
                flip_call_result.message_result().is_ok(),
                "Messages now return a `Result`, which should be `Ok` here."
            );

            let flipped_value = client
                .call_dry_run(&ink_e2e::alice(), &get, 0, None)
                .await
                .return_value();
            assert!(flipped_value != initial_value);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_message_error_reverts_state(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = FlipperRef::new_default();
            let contract_acc_id = client
                .instantiate("integration_flipper", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<FlipperRef>(contract_acc_id.clone())
                .call(|contract| contract.get());
            let initial_value = client
                .call_dry_run(&ink_e2e::bob(), &get, 0, None)
                .await
                .return_value();

            let err_flip = build_message::<FlipperRef>(contract_acc_id)
                .call(|contract| contract.err_flip());
            let err_flip_call_result =
                client.call(&ink_e2e::bob(), err_flip, 0, None).await;

            assert!(matches!(
                err_flip_call_result,
                Err(ink_e2e::Error::CallExtrinsic(_))
            ));

            let flipped_value = client
                .call_dry_run(&ink_e2e::bob(), &get, 0, None)
                .await
                .return_value();
            assert!(flipped_value == initial_value);

            Ok(())
        }
    }
}
