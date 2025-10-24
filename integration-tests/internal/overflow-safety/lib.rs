#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod overflow_safety {
    #[ink(storage)]
    pub struct OverflowSafety {}

    impl OverflowSafety {
        /// Creates a new smart contract.
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {}
        }

        /// Adds the given values.
        ///
        /// # Note
        ///
        /// Should panic on overflow.
        #[ink(message)]
        pub fn add(&self, a: u8, b: u8) -> u8 {
            a + b
        }

        /// Subtracts the given values.
        ///
        /// # Note
        ///
        /// Should panic on overflow.
        #[ink(message)]
        pub fn sub(&self, a: u8, b: u8) -> u8 {
            a - b
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn add_no_overflow_works() {
            let overflow_safety = OverflowSafety::new();
            assert_eq!(overflow_safety.add(100u8, 50u8), 150u8);
        }

        #[ink::test]
        #[should_panic(expected = "attempt to add with overflow")]
        fn add_with_overflow_panics() {
            let overflow_safety = OverflowSafety::new();
            overflow_safety.add(u8::MAX, 1u8);
        }

        #[ink::test]
        fn sub_no_overflow_works() {
            let overflow_safety = OverflowSafety::new();
            assert_eq!(overflow_safety.sub(100u8, 50u8), 50u8);
        }

        #[ink::test]
        #[should_panic(expected = "attempt to subtract with overflow")]
        fn sub_with_overflow_panics() {
            let overflow_safety = OverflowSafety::new();
            overflow_safety.sub(u8::MIN, 1u8);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn add_no_overflow_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = OverflowSafetyRef::new();
            let contract = client
                .instantiate("overflow_safety", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<OverflowSafety>();

            // when
            let add = call_builder.add(100u8, 50u8);
            let add_res = client
                .call(&ink_e2e::bob(), &add)
                .submit()
                .await
                .expect("add failed");
            assert_eq!(add_res.return_value(), 150u8);

            Ok(())
        }

        #[ink_e2e::test]
        async fn add_with_overflow_reverts<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = OverflowSafetyRef::new();
            let contract = client
                .instantiate("overflow_safety", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<OverflowSafety>();

            // when
            let add = call_builder.add(u8::MAX, 1u8);
            let add_res = client.call(&ink_e2e::bob(), &add).submit().await;
            assert!(matches!(add_res, Err(ink_e2e::Error::CallExtrinsic(_, _))));

            Ok(())
        }

        #[ink_e2e::test]
        async fn sub_no_overflow_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = OverflowSafetyRef::new();
            let contract = client
                .instantiate("overflow_safety", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<OverflowSafety>();

            // when
            let sub = call_builder.sub(100u8, 50u8);
            let sub_res = client
                .call(&ink_e2e::bob(), &sub)
                .submit()
                .await
                .expect("add failed");
            assert_eq!(sub_res.return_value(), 50u8);

            Ok(())
        }

        #[ink_e2e::test]
        async fn sub_with_overflow_reverts<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = OverflowSafetyRef::new();
            let contract = client
                .instantiate("overflow_safety", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<OverflowSafety>();

            // when
            let sub = call_builder.sub(u8::MIN, 1u8);
            let sub_res = client.call(&ink_e2e::bob(), &sub).submit().await;
            assert!(matches!(sub_res, Err(ink_e2e::Error::CallExtrinsic(_, _))));

            Ok(())
        }
    }
}
