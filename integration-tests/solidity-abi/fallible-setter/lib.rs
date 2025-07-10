#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[derive(ink::SolErrorDecode, ink::SolErrorEncode)]
pub struct SetFailed;

#[ink::contract]
pub mod fallible_setter {
    use super::SetFailed;

    #[ink(storage)]
    pub struct FallibleSetter {
        value: bool,
    }

    impl FallibleSetter {
        /// Creates a new fallible setter smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Sets the value of the FallibleSetter's boolean.
        /// Returns an error if the given value is the same as the current value.
        #[ink(message)]
        pub fn try_set(&mut self, value: bool) -> Result<(), SetFailed> {
            if self.value == value {
                return Err(SetFailed);
            }

            self.value = value;
            Ok(())
        }

        /// Returns the current value of the FallibleSetter's boolean.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn it_works() {
            let mut fallible_setter = FallibleSetter::new(false);
            assert!(!fallible_setter.get());
            let res = fallible_setter.try_set(true);
            assert!(res.is_ok());
            assert!(fallible_setter.get());
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
            let mut constructor = FallibleSetterRef::new(false);
            let contract = client
                .instantiate("fallible_setter", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<FallibleSetter>();

            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::bob(), &get).submit().await?;
            assert!(!get_res.return_value());

            // when
            let set = call_builder.try_set(true);
            let set_res = client
                .call(&ink_e2e::bob(), &set)
                .submit()
                .await
                .expect("set failed");
            assert!(set_res.return_value().is_ok());

            // then
            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(get_res.return_value());

            Ok(())
        }
    }
}
