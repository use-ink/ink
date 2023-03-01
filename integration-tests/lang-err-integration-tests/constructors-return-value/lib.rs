#![cfg_attr(not(feature = "std"), no_std)]

pub use self::constructors_return_value::{
    ConstructorError,
    ConstructorsReturnValue,
    ConstructorsReturnValueRef,
};

#[ink::contract]
pub mod constructors_return_value {
    #[ink(storage)]
    pub struct ConstructorsReturnValue {
        value: bool,
    }

    #[derive(scale::Encode, scale::Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ConstructorError;

    impl ConstructorsReturnValue {
        /// Infallible constructor
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Fallible constructor
        #[ink(constructor)]
        pub fn try_new(succeed: bool) -> Result<Self, ConstructorError> {
            if succeed {
                Ok(Self::new(true))
            } else {
                Err(ConstructorError)
            }
        }

        /// A constructor which reverts and fills the output buffer with an erroneously encoded
        /// return value.
        #[ink(constructor)]
        pub fn revert_new(_init_value: bool) -> Self {
            ink::env::return_value::<ink::ConstructorResult<AccountId>>(
                ink::env::ReturnFlags::new_with_reverted(true),
                &Ok(AccountId::from([0u8; 32])),
            );
            unreachable!()
        }

        /// A constructor which reverts and fills the output buffer with an erroneously encoded
        /// return value.
        #[ink(constructor)]
        pub fn try_revert_new(init_value: bool) -> Result<Self, ConstructorError> {
            let value = if init_value {
                Ok(Ok(AccountId::from([0u8; 32])))
            } else {
                Err(ink::LangError::CouldNotReadInput)
            };

            ink::env::return_value::<
                ink::ConstructorResult<Result<AccountId, ConstructorError>>,
            >(ink::env::ReturnFlags::new_with_reverted(true), &value);
            unreachable!();
        }

        /// Returns the current value of the contract storage.
        #[ink(message)]
        pub fn get_value(&self) -> bool {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::ConstructorsReturnValue as Contract;
        use std::any::TypeId;

        #[test]
        #[allow(clippy::assertions_on_constants)]
        fn infallible_constructor_reflection() {
            const ID: u32 =
                <Contract as ::ink::reflect::ContractDispatchableConstructors<
                    {
                        <Contract as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                    },
                >>::IDS[0];

            assert!(
                !<Contract as ::ink::reflect::DispatchableConstructorInfo<{ ID }>>::IS_RESULT,
            );
            assert_eq!(
                TypeId::of::<
                    <Contract as ::ink::reflect::DispatchableConstructorInfo<{ ID }>>::Error,
                >(),
                TypeId::of::<&()>(),
            )
        }

        #[test]
        #[allow(clippy::assertions_on_constants)]
        fn fallible_constructor_reflection() {
            const ID: u32 =
                <Contract as ::ink::reflect::ContractDispatchableConstructors<
                    {
                        <Contract as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                    },
                >>::IDS[1];

            assert!(
                <Contract as ::ink::reflect::DispatchableConstructorInfo<{ ID }>>::IS_RESULT,
            );
            assert_eq!(
                TypeId::of::<
                    <Contract as ::ink::reflect::DispatchableConstructorInfo<{ ID }>>::Error,
                >(),
                TypeId::of::<super::ConstructorError>(),
            )
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::ConstructorsReturnValueRef;
        use scale::Decode as _;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_infallible_constructor(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = ConstructorsReturnValueRef::new(true);
            let infallible_constructor_result = client
                .instantiate_dry_run(
                    "constructors_return_value",
                    &ink_e2e::alice(),
                    constructor,
                    0,
                    None,
                )
                .await
                .result
                .expect("Instantiate dry run should succeed");

            let data = infallible_constructor_result.result.data;
            let decoded_result = Result::<(), ink::LangError>::decode(&mut &data[..])
                .expect("Failed to decode constructor Result");
            assert!(
                decoded_result.is_ok(),
                "Constructor dispatch should have succeeded"
            );

            let constructor = ConstructorsReturnValueRef::new(true);
            let success = client
                .instantiate(
                    "constructors_return_value",
                    &ink_e2e::alice(),
                    constructor,
                    0,
                    None,
                )
                .await
                .is_ok();

            assert!(success, "Contract created successfully");

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_fallible_constructor_succeed(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = ConstructorsReturnValueRef::try_new(true);
            let result = client
                .instantiate_dry_run(
                    "constructors_return_value",
                    &ink_e2e::bob(),
                    constructor,
                    0,
                    None,
                )
                .await
                .result
                .expect("Instantiate dry run should succeed");

            let decoded_result = Result::<
                Result<(), super::ConstructorError>,
                ink::LangError,
            >::decode(&mut &result.result.data[..])
            .expect("Failed to decode fallible constructor Result");

            assert!(
                decoded_result.is_ok(),
                "Constructor dispatch should have succeeded"
            );

            assert!(
                decoded_result.unwrap().is_ok(),
                "Fallible constructor should have succeeded"
            );

            let constructor = ConstructorsReturnValueRef::try_new(true);
            let contract_acc_id = client
                .instantiate(
                    "constructors_return_value",
                    &ink_e2e::bob(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            let get =
                ink_e2e::build_message::<ConstructorsReturnValueRef>(contract_acc_id)
                    .call(|contract| contract.get_value());
            let value = client
                .call_dry_run(&ink_e2e::bob(), &get, 0, None)
                .await
                .return_value();

            assert_eq!(
                true, value,
                "Contract success should write to contract storage"
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_fallible_constructor_fails(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = ConstructorsReturnValueRef::try_new(false);

            let result = client
                .instantiate_dry_run(
                    "constructors_return_value",
                    &ink_e2e::charlie(),
                    constructor,
                    0,
                    None,
                )
                .await
                .result
                .expect("Instantiate dry run should succeed");

            let decoded_result = Result::<
                Result<(), super::ConstructorError>,
                ink::LangError,
            >::decode(&mut &result.result.data[..])
            .expect("Failed to decode fallible constructor Result");

            assert!(
                decoded_result.is_ok(),
                "Constructor dispatch should have succeeded"
            );

            assert!(
                decoded_result.unwrap().is_err(),
                "Fallible constructor should have failed"
            );

            let constructor = ConstructorsReturnValueRef::try_new(false);
            let result = client
                .instantiate(
                    "constructors_return_value",
                    &ink_e2e::charlie(),
                    constructor,
                    0,
                    None,
                )
                .await;

            assert!(
                matches!(result, Err(ink_e2e::Error::InstantiateExtrinsic(_))),
                "Constructor should fail"
            );

            Ok(())
        }
    }
}
