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

    #[derive(scale::Encode, scale::Decode)]
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

        /// Invoke the fallible constructor via a contract ref
        #[ink(message)]
        pub fn call_fallible_constructor(
            &self,
            _succeed: bool,
        ) -> Result<(), ConstructorError> {
            todo!()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::ConstructorsReturnValue as Contract;
        use std::any::TypeId;

        #[test]
        fn infallible_constructor_reflection() {
            const ID: u32 =
                <Contract as ::ink::reflect::ContractDispatchableConstructors<
                    {
                        <Contract as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                    },
                >>::IDS[0];

            assert_eq!(
                <Contract as ::ink::reflect::DispatchableConstructorInfo<{ ID }>>::IS_RESULT,
                false
            );
            assert_eq!(
                TypeId::of::<
                    <Contract as ::ink::reflect::DispatchableConstructorInfo<{ ID }>>::Error,
                >(),
                TypeId::of::<&()>(),
            )
        }

        #[test]
        fn fallible_constructor_reflection() {
            const ID: u32 =
                <Contract as ::ink::reflect::ContractDispatchableConstructors<
                    {
                        <Contract as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                    },
                >>::IDS[1];

            assert_eq!(
                <Contract as ::ink::reflect::DispatchableConstructorInfo<{ ID }>>::IS_RESULT,
                true
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
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_infallible_constructor(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = constructors_return_value::constructors::new(true);

            let infallible_constructor_result = client
                .instantiate_dry_run(&ink_e2e::alice(), &constructor, 0, None)
                .await
                .result
                .expect("Instantiate dry run should succeed");

            assert!(
                infallible_constructor_result.result.data.is_empty(),
                "Infallible constructor should return no data"
            );

            let success = client
                .instantiate(&mut ink_e2e::alice(), constructor, 0, None)
                .await
                .is_ok();

            assert!(success, "Contract created successfully");

            Ok(())
        }
    }
}
