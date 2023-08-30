#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod flipper {
    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new_default() -> Self {
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
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(backend = "runtime-only")]
        async fn it_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let constructor = FlipperRef::new(false);
            let contract = client
                .instantiate(
                    "e2e-runtime-only-backend",
                    &ink_e2e::alice(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed");

            let mut call = contract.call::<Flipper>();

            // when
            let _flip_res = client
                .call(&ink_e2e::bob(), &call.flip(), 0, None)
                .await
                .expect("flip failed");

            // then
            let get_res = client
                .call(&ink_e2e::bob(), &call.get(), 0, None)
                .await
                .expect("get failed");

            assert!(matches!(get_res.return_value(), true));

            Ok(())
        }

        /// Just instantiate a contract using non-default runtime.
        #[ink_e2e::test(backend = "runtime-only", runtime = drink::runtime::MinimalRuntime)]
        async fn custom_runtime<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            client
                .instantiate(
                    "e2e-runtime-only-backend",
                    &ink_e2e::alice(),
                    FlipperRef::new(false),
                    0,
                    None,
                )
                .await
                .expect("instantiate failed");

            Ok(())
        }
    }
}
