#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod delegatecall_bug {
    use ink::storage::Mapping;

    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        DefaultEnvironment,
    };

    #[ink(storage)]
    pub struct DelegateCallBug {
        values: Mapping<AccountId, Balance>,
        value: bool,
    }

    impl DelegateCallBug {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            let v = Mapping::new();
            Self {
                value: init_value,
                values: v,
            }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        /// Flips the current value using delegatecall
        #[allow(clippy::result_unit_err)]
        #[ink(message)]
        pub fn flip_delegate(&mut self, hash: Hash) -> Result<(), ()> {
            let selector = ink::selector_bytes!("flip");
            let _ = build_call::<DefaultEnvironment>()
                .delegate(hash)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .try_invoke().map_err(|_| ())?;
            Ok(())
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

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_storage_not_mutated(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
                .await;

            let constructor = DelegateCallBugRef::new(false);
            let call_builder = client
                .instantiate("delegatecall-bug", &origin, constructor, 0, None)
                .await
                .expect("instantiate failed");
            let mut call_builder_call = call_builder.call::<DelegateCallBug>();

            let code_hash = client
                .upload("key-reproducer", &origin, None)
                .await
                .expect("upload `key-reproducer` failed")
                .code_hash;

            let call_delegate = call_builder_call.flip_delegate(code_hash);
            let result = client
                .call(&origin, &call_delegate, 0, None)
                .await
                .expect("Client failed to call `call_builder::flip`.")
                .return_value();
            // indicates that delegate call that mutates the value hsa been executed correctly
            assert!(result.is_ok());

            let expected_value = true;
            let call = call_builder.call::<DelegateCallBug>();

            let call_get = call.get();
            let call_get_result = client
                .call_dry_run(&origin, &call_get, 0, None)
                .await
                .return_value();

            // This fails
            assert_eq!(
                call_get_result, expected_value,
                "Decoded an unexpected value from the call."
            );

            Ok(())
        }
    }
}
