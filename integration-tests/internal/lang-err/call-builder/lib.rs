//! # Integration Tests for `LangError`
//!
//! This contract is used to ensure that the behavior around `LangError`s works as
//! expected.
//!
//! In particular, it exercises the codepaths that stem from the usage of the
//! [`CallBuilder`](`ink::env::call::CallBuilder`) and
//! [`CreateBuilder`](`ink::env::call::CreateBuilder`) structs.
//!
//! This differs from the codepath used by external tooling, such as `cargo-contract` or
//! the `Contracts-UI` which instead depend on methods from `pallet-revive` which are
//! exposed via RPC.
//!
//! Note that during testing we make use of ink!'s end-to-end testing features, so ensure
//! that you have a node which includes `pallet-revive` running alongside your tests.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod call_builder {
    use constructors_return_value::ConstructorsReturnValueRef;
    use ink::{
        H256,
        env::{
            DefaultEnvironment,
            call::{
                ExecutionInput,
                Selector,
                build_call,
            },
        },
    };

    #[ink(storage)]
    #[derive(Default)]
    pub struct CallBuilderTest {}

    impl CallBuilderTest {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Call a contract using the `CallBuilder`.
        ///
        /// Since we can't use the `CallBuilder` in a test environment directly we need
        /// this wrapper to test things like crafting calls with invalid
        /// selectors.
        ///
        /// We also wrap the output in an `Option` since we can't return a `Result`
        /// directly from a contract message without erroring out ourselves.
        #[ink(message)]
        pub fn call(
            &mut self,
            address: Address,
            selector: [u8; 4],
        ) -> Option<ink::LangError> {
            let result = build_call::<DefaultEnvironment>()
                .call(address)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .try_invoke()
                .expect("Error from `pallet-revive`.");

            match result {
                Ok(_) => None,
                Err(e @ ink::LangError::CouldNotReadInput) => Some(e),
                Err(_) => {
                    unimplemented!("No other `LangError` variants exist at the moment.")
                }
            }
        }

        /// Call a contract using the `CallBuilder`.
        ///
        /// Since we can't use the `CallBuilder` in a test environment directly we need
        /// this wrapper to test things like crafting calls with invalid
        /// selectors.
        ///
        /// This message does not allow the caller to handle any `LangErrors`, for that
        /// use the `call` message instead.
        #[ink(message)]
        pub fn invoke(&mut self, address: Address, selector: [u8; 4]) {
            use ink::env::call::build_call;

            build_call::<DefaultEnvironment>()
                .call(address)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .invoke()
        }

        /// Instantiate a contract using the `CreateBuilder`.
        ///
        /// Since we can't use the `CreateBuilder` in a test environment directly we need
        /// this wrapper to test things like crafting calls with invalid
        /// selectors.
        ///
        /// We also wrap the output in an `Option` since we can't return a `Result`
        /// directly from a contract message without erroring out ourselves.
        #[ink(message)]
        pub fn call_instantiate(
            &mut self,
            code_hash: H256,
            selector: [u8; 4],
            init_value: bool,
        ) -> Option<ink::LangError> {
            let mut params = ConstructorsReturnValueRef::new(init_value)
                .code_hash(code_hash)
                .endowment(0.into())
                .salt_bytes(Some([1u8; 32]))
                .params();

            params.update_selector(Selector::new(selector));

            let result = params
                .try_instantiate()
                .expect("Error from the `pallet-revive`.");

            match result {
                Ok(_) => None,
                Err(e @ ink::LangError::CouldNotReadInput) => Some(e),
                Err(_) => {
                    unimplemented!("No other `LangError` variants exist at the moment.")
                }
            }
        }

        /// Attempt to instantiate a contract using the `CreateBuilder`.
        ///
        /// Since we can't use the `CreateBuilder` in a test environment directly we need
        /// this wrapper to test things like crafting calls with invalid
        /// selectors.
        ///
        /// We also wrap the output in an `Option` since we can't return a `Result`
        /// directly from a contract message without erroring out ourselves.
        #[ink(message)]
        pub fn call_instantiate_fallible(
            &mut self,
            code_hash: H256,
            selector: [u8; 4],
            init_value: bool,
        ) -> Option<
            Result<
                Result<Address, constructors_return_value::ConstructorError>,
                ink::LangError,
            >,
        > {
            let mut params = ConstructorsReturnValueRef::try_new(init_value)
                .code_hash(code_hash)
                .endowment(0.into())
                .salt_bytes(Some([1u8; 32]))
                .params();

            params.update_selector(Selector::new(selector));

            let lang_result = params
                .try_instantiate()
                .expect("Error from `pallet-revive`.");

            Some(lang_result.map(|contract_result| {
                contract_result.map(|inner| ink::ToAddr::to_addr(&inner))
            }))
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{
            ChainBackend,
            ContractsBackend,
        };
        use integration_flipper::{
            Flipper,
            FlipperRef,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_invalid_message_selector_can_be_handled<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
                .await;

            let mut constructor = CallBuilderTestRef::new();
            let contract = client
                .instantiate("call_builder", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CallBuilderTest>();

            let mut flipper_constructor = FlipperRef::new_default();
            let flipper = client
                .instantiate("integration_flipper", &origin, &mut flipper_constructor)
                .submit()
                .await
                .expect("instantiate `flipper` failed");
            let flipper_call = flipper.call_builder::<Flipper>();

            let flipper_get = flipper_call.get();
            let get_call_result = client.call(&origin, &flipper_get).dry_run().await?;
            let initial_value = get_call_result.return_value();

            let selector = ink::selector_bytes!("invalid_selector");
            let call = call_builder.call(flipper.addr, selector);
            let call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect("Calling `call_builder::call` failed");

            let flipper_result = call_result.return_value();

            assert!(matches!(
                flipper_result,
                Some(ink::LangError::CouldNotReadInput)
            ));

            let flipper_get = flipper_call.get();
            let get_call_result = client.call(&origin, &flipper_get).dry_run().await?;
            let flipped_value = get_call_result.return_value();
            assert!(flipped_value == initial_value);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_invalid_message_selector_panics_on_invoke<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let mut constructor = CallBuilderTestRef::new();
            let contract = client
                .instantiate("call_builder", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CallBuilderTest>();

            let mut flipper_constructor = FlipperRef::new_default();
            let flipper = client
                .instantiate(
                    "integration_flipper",
                    &ink_e2e::bob(),
                    &mut flipper_constructor,
                )
                .submit()
                .await
                .expect("instantiate `flipper` failed");

            // Since `LangError`s can't be handled by the `CallBuilder::invoke()` method
            // we expect this to panic.
            let invalid_selector = [0x00, 0x00, 0x00, 0x00];
            let call = call_builder.invoke(flipper.addr, invalid_selector);
            let call_result = client.call(&ink_e2e::bob(), &call).dry_run().await?;
            assert!(call_result.did_revert());
            let err_msg = String::from_utf8_lossy(call_result.return_data());
            assert!(
                err_msg.contains("Cross-contract call failed with CouldNotReadInput")
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_create_builder_works_with_valid_selector<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::bob(), 10_000_000_000_000)
                .await;

            let mut constructor = CallBuilderTestRef::new();
            let contract = client
                .instantiate("call_builder", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CallBuilderTest>();

            let code_hash = client
                .upload("constructors_return_value", &origin)
                .submit()
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("new");
            let init_value = true;
            let call = call_builder.call_instantiate(code_hash, selector, init_value);
            let call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect("Client failed to call `call_builder::call_instantiate`.")
                .return_value();

            assert!(
                call_result.is_none(),
                "Call using valid selector failed, when it should've succeeded."
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_create_builder_fails_with_invalid_selector<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::bob(), 10_000_000_000_000)
                .await;

            let mut constructor = CallBuilderTestRef::new();
            let contract = client
                .instantiate("call_builder", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CallBuilderTest>();

            let code_hash = client
                .upload("constructors_return_value", &origin)
                .submit()
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("invalid_selector");
            let init_value = true;
            let call = call_builder.call_instantiate(code_hash, selector, init_value);
            let call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect("Client failed to call `call_builder::call_instantiate`.")
                .return_value();

            assert!(
                matches!(call_result, Some(ink::LangError::CouldNotReadInput)),
                "Call using invalid selector succeeded, when it should've failed."
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_create_builder_with_infallible_revert_constructor_encodes_ok<
            Client: E2EBackend,
        >(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::bob(), 10_000_000_000_000)
                .await;

            let mut constructor = CallBuilderTestRef::new();
            let contract = client
                .instantiate("call_builder", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CallBuilderTest>();

            let code_hash = client
                .upload("constructors_return_value", &origin)
                .submit()
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("revert_new");
            let init_value = false;
            let call = call_builder.call_instantiate(code_hash, selector, init_value);

            let call_result = client.call(&origin, &call).dry_run().await?;
            assert!(call_result.did_revert());
            let err_msg = String::from_utf8_lossy(call_result.return_data());
            // The callee reverted, but did not encode an error in the output buffer.
            // So the output buffer couldn't be decoded.
            assert!(err_msg.contains("Decode(Error)"));

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_create_builder_can_handle_fallible_constructor_success<
            Client: E2EBackend,
        >(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::bob(), 10_000_000_000_000)
                .await;

            let mut constructor = CallBuilderTestRef::new();
            let contract = client
                .instantiate("call_builder", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CallBuilderTest>();

            let code_hash = client
                .upload("constructors_return_value", &origin)
                .submit()
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_new");
            let init_value = true;
            let call =
                call_builder.call_instantiate_fallible(code_hash, selector, init_value);
            let call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect("Calling `call_builder::call_instantiate_fallible` failed")
                .return_value();

            assert!(
                matches!(call_result, Some(Ok(_))),
                "Call to fallible constructor failed, when it should have succeeded."
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_create_builder_can_handle_fallible_constructor_error<
            Client: E2EBackend,
        >(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::bob(), 10_000_000_000_000)
                .await;

            let mut constructor = CallBuilderTestRef::new();
            let contract = client
                .instantiate("call_builder", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CallBuilderTest>();

            let code_hash = client
                .upload("constructors_return_value", &origin)
                .submit()
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_new");
            let init_value = false;
            let call =
                call_builder.call_instantiate_fallible(code_hash, selector, init_value);
            let call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect("Calling `call_builder::call_instantiate_fallible` failed")
                .return_value();

            let contract_result = call_result
                .unwrap()
                .expect("Dispatching `constructors_return_value::try_new` failed.");

            assert!(
                matches!(
                    contract_result,
                    Err(constructors_return_value::ConstructorError)
                ),
                "Got an unexpected error from the contract."
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_create_builder_with_fallible_revert_constructor_encodes_ok<
            Client: E2EBackend,
        >(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::bob(), 10_000_000_000_000)
                .await;

            let mut constructor = CallBuilderTestRef::new();
            let contract = client
                .instantiate("call_builder", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CallBuilderTest>();

            let code_hash = client
                .upload("constructors_return_value", &origin)
                .submit()
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_revert_new");
            let init_value = true;
            let call =
                call_builder.call_instantiate_fallible(code_hash, selector, init_value);
            let call_result = client.call(&origin, &call).dry_run().await?;
            assert!(call_result.did_revert());
            let err_msg = String::from_utf8_lossy(call_result.return_data());
            // The callee reverted, but did not encode an error in the output buffer.
            // So the output buffer couldn't be decoded.
            assert!(err_msg.contains("Decode(Error)"));

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_create_builder_with_fallible_revert_constructor_encodes_err<
            Client: E2EBackend,
        >(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::bob(), 10_000_000_000_000)
                .await;

            let mut constructor = CallBuilderTestRef::new();
            let contract = client
                .instantiate("call_builder", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CallBuilderTest>();

            let code_hash = client
                .upload("constructors_return_value", &origin)
                .submit()
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_revert_new");
            let init_value = false;
            let call =
                call_builder.call_instantiate_fallible(code_hash, selector, init_value);
            let call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect(
                    "Client failed to call `call_builder::call_instantiate_fallible`.",
                )
                .return_value();

            assert!(
                matches!(call_result, Some(Err(ink::LangError::CouldNotReadInput))),
                "The callee manually encoded `CouldNotReadInput` to the output buffer, we should've
                 gotten that back."
            );

            Ok(())
        }
    }
}
