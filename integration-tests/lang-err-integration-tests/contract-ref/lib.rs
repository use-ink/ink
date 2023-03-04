#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod contract_ref {
    use integration_flipper::FlipperRef;

    #[ink(storage)]
    pub struct ContractRef {
        flipper: FlipperRef,
    }

    impl ContractRef {
        #[ink(constructor)]
        pub fn new(version: u32, flipper_code_hash: Hash) -> Self {
            let salt = version.to_le_bytes();
            let flipper = FlipperRef::new_default()
                .endowment(0)
                .code_hash(flipper_code_hash)
                .salt_bytes(salt)
                .instantiate();

            Self { flipper }
        }

        #[ink(constructor)]
        pub fn try_new(version: u32, flipper_code_hash: Hash, succeed: bool) -> Self {
            let salt = version.to_le_bytes();
            let flipper = FlipperRef::try_new(succeed)
                .endowment(0)
                .code_hash(flipper_code_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "Received an error from the Flipper constructor while instantiating \
                         Flipper {error:?}"
                    )
                });

            Self { flipper }
        }

        #[ink(message)]
        pub fn flip(&mut self) {
            self.flipper.flip();
        }

        #[ink(message)]
        pub fn flip_check(&mut self) {
            self.flipper
                .try_flip()
                .expect("The ink! codegen should've produced a valid call.");
        }

        #[ink(message)]
        pub fn get(&mut self) -> bool {
            self.flipper.get()
        }

        #[ink(message)]
        pub fn get_check(&mut self) -> bool {
            self.flipper
                .try_get()
                .expect("The ink! codegen should've produced a valid call.")
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::ContractRefRef;
        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_ref_can_flip_correctly(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let flipper_hash = client
                .upload("integration_flipper", &ink_e2e::alice(), None)
                .await
                .expect("uploading `flipper` failed")
                .code_hash;

            let constructor = ContractRefRef::new(0, flipper_hash);
            let contract_acc_id = client
                .instantiate("contract_ref", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get_check = build_message::<ContractRefRef>(contract_acc_id.clone())
                .call(|contract| contract.get_check());
            let get_call_result = client
                .call_dry_run(&ink_e2e::alice(), &get_check, 0, None)
                .await;

            let initial_value = get_call_result.return_value();

            let flip_check = build_message::<ContractRefRef>(contract_acc_id.clone())
                .call(|contract| contract.flip_check());
            let flip_call_result = client
                .call(&ink_e2e::alice(), flip_check, 0, None)
                .await
                .expect("Calling `flip` failed");
            assert!(
                flip_call_result.message_result().is_ok(),
                "Messages now return a `Result`, which should be `Ok` here."
            );

            let get_call_result = client
                .call_dry_run(&ink_e2e::alice(), &get_check, 0, None)
                .await;
            let flipped_value = get_call_result.return_value();
            assert!(flipped_value != initial_value);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_fallible_ref_can_be_instantiated(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let flipper_hash = client
                .upload("integration_flipper", &ink_e2e::bob(), None)
                .await
                .expect("uploading `flipper` failed")
                .code_hash;

            let succeed = true;
            let constructor = ContractRefRef::try_new(0, flipper_hash, succeed);
            let contract_acc_id = client
                .instantiate("contract_ref", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get_check = build_message::<ContractRefRef>(contract_acc_id.clone())
                .call(|contract| contract.get_check());
            let get_call_result = client
                .call_dry_run(&ink_e2e::bob(), &get_check, 0, None)
                .await;
            let initial_value = get_call_result.return_value();

            assert!(initial_value);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_fallible_ref_fails_to_be_instantiated(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let flipper_hash = client
                .upload("integration_flipper", &ink_e2e::charlie(), None)
                .await
                .expect("uploading `flipper` failed")
                .code_hash;

            let succeed = false;
            let constructor = ContractRefRef::try_new(0, flipper_hash, succeed);
            let instantiate_result = client
                .instantiate("contract_ref", &ink_e2e::charlie(), constructor, 0, None)
                .await;

            assert!(
                instantiate_result.is_err(),
                "Call execution should've failed, but didn't."
            );

            let contains_err_msg = match instantiate_result.unwrap_err() {
                ink_e2e::Error::InstantiateDryRun(dry_run) => {
                    String::from_utf8_lossy(&dry_run.debug_message).contains(
                        "Received an error from the Flipper constructor while instantiating Flipper FlipperError"
                    )
                }
                _ => false,
            };
            assert!(
                contains_err_msg,
                "Call execution failed for an unexpected reason."
            );

            Ok(())
        }
    }
}
