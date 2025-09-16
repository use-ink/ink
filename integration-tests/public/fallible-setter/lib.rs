#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::error]
#[derive(Debug, PartialEq, Eq)]
/// Equivalent to multiple Solidity custom errors, one for each variant.
pub enum Error {
    /// Error when `value > 100`
    TooLarge,
    /// Error when `value == self.value`
    NoChange,
}

#[ink::contract]
pub mod fallible_setter {
    use super::Error;

    #[ink(storage)]
    pub struct FallibleSetter {
        value: u8,
    }

    impl FallibleSetter {
        /// Creates a new fallible setter smart contract initialized with the given value.
        /// Returns an error if `init_value > 100`.
        #[ink(constructor)]
        pub fn new(init_value: u8) -> Result<Self, Error> {
            if init_value > 100 {
                return Err(Error::TooLarge)
            }
            Ok(Self { value: init_value })
        }

        /// Sets the value of the FallibleSetter's `u8`.
        /// Returns an appropriate error if any of the following is true:
        /// - `value == self.value`
        /// - `init_value > 100`
        #[ink(message)]
        pub fn try_set(&mut self, value: u8) -> Result<(), Error> {
            if self.value == value {
                return Err(Error::NoChange);
            }

            if value > 100 {
                return Err(Error::TooLarge);
            }

            self.value = value;
            Ok(())
        }

        /// Returns the current value of the FallibleSetter's `u8`.
        #[ink(message)]
        pub fn get(&self) -> u8 {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn it_works() {
            // given
            let mut fallible_setter = FallibleSetter::new(0).expect("init failed");
            assert_eq!(fallible_setter.get(), 0);

            // when
            let res = fallible_setter.try_set(1);
            assert!(res.is_ok());

            // when
            let res = fallible_setter.try_set(1);
            assert_eq!(res, Err(Error::NoChange));

            // when
            let res = fallible_setter.try_set(101);
            assert_eq!(res, Err(Error::TooLarge));

            // then
            assert_eq!(fallible_setter.get(), 1);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn it_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let mut constructor = FallibleSetterRef::new(0);
            let contract = client
                .instantiate("fallible_setter", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<FallibleSetter>();

            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::bob(), &get).submit().await?;
            assert_eq!(get_res.return_value(), 0);

            // when
            let set = call_builder.try_set(1);
            let set_res = client
                .call(&ink_e2e::bob(), &set)
                .submit()
                .await
                .expect("set failed");
            assert!(set_res.return_value().is_ok());

            // when
            let set = call_builder.try_set(1);
            let set_res = client.call(&ink_e2e::bob(), &set).submit().await;
            assert!(matches!(set_res, Err(ink_e2e::Error::CallExtrinsic(_, _))));

            // when
            let set = call_builder.try_set(101);
            let set_res = client.call(&ink_e2e::bob(), &set).submit().await;
            assert!(matches!(set_res, Err(ink_e2e::Error::CallExtrinsic(_, _))));

            // then
            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert_eq!(get_res.return_value(), 1);

            Ok(())
        }
    }
}
