#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract_ref {
    use integration_flipper::FlipperRef;

    #[ink(storage)]
    pub struct ContractRef {
        flipper: FlipperRef,
    }

    impl ContractRef {
        #[ink(constructor)]
        pub fn new(version: u32, flipper_code_hash: ink::H256) -> Self {
            let flipper = FlipperRef::new_default()
                .endowment(0.into())
                .code_hash(flipper_code_hash)
                .salt_bytes(salt_from_version(version))
                .instantiate();

            Self { flipper }
        }

        #[ink(constructor)]
        pub fn try_new(
            version: u32,
            flipper_code_hash: ink::H256,
            succeed: bool,
        ) -> Self {
            let flipper = FlipperRef::try_new(succeed)
                .endowment(0.into())
                .code_hash(flipper_code_hash)
                .salt_bytes(salt_from_version(version))
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

    fn salt_from_version(version: u32) -> Option<[u8; 32]> {
        let version: [u8; 4] = version.to_le_bytes();
        let mut salt: [u8; 32] = [0u8; 32];
        salt[..4].copy_from_slice(&version);
        Some(salt)
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_ref_can_flip_correctly<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let flipper_hash = client
                .upload("integration_flipper", &ink_e2e::alice())
                .submit()
                .await
                .expect("uploading `flipper` failed")
                .code_hash;

            let mut constructor = ContractRefRef::new(0, flipper_hash);
            let contract_ref = client
                .instantiate("contract_ref", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract_ref.call_builder::<ContractRef>();

            let get_check = call_builder.get_check();
            let get_call_result =
                client.call(&ink_e2e::alice(), &get_check).dry_run().await?;

            let initial_value = get_call_result.return_value();

            let flip_check = call_builder.flip_check();
            let flip_call_result = client
                .call(&ink_e2e::alice(), &flip_check)
                .submit()
                .await
                .expect("Calling `flip` failed");
            assert!(
                flip_call_result.message_result().is_ok(),
                "Messages now return a `Result`, which should be `Ok` here."
            );

            let get_call_result =
                client.call(&ink_e2e::alice(), &get_check).dry_run().await?;
            let flipped_value = get_call_result.return_value();
            assert!(flipped_value != initial_value);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_fallible_ref_can_be_instantiated<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let flipper_hash: ink::H256 = client
                .upload("integration_flipper", &ink_e2e::bob())
                .submit()
                .await
                .expect("uploading `flipper` failed")
                .code_hash;

            let succeed = true;
            let mut constructor = ContractRefRef::try_new(0, flipper_hash, succeed);
            let contract_ref = client
                .instantiate("contract_ref", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiating `contract_ref failed");
            let mut call_builder = contract_ref.call_builder::<ContractRef>();

            let get_check = call_builder.get_check();
            let get_call_result =
                client.call(&ink_e2e::bob(), &get_check).dry_run().await?;
            let initial_value = get_call_result.return_value();

            assert!(initial_value);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_fallible_ref_fails_to_be_instantiated<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let flipper_hash = client
                .upload("integration_flipper", &ink_e2e::charlie())
                .submit()
                .await
                .expect("uploading `flipper` failed")
                .code_hash;

            let succeed = false;
            let mut constructor = ContractRefRef::try_new(0, flipper_hash, succeed);
            let instantiate_result = client
                .instantiate("contract_ref", &ink_e2e::charlie(), &mut constructor)
                .dry_run()
                .await?;

            let err_msg = String::from_utf8_lossy(instantiate_result.return_data());
            assert!(instantiate_result.did_revert());
            assert!(err_msg.contains("Received an error from the Flipper constructor while instantiating Flipper FlipperError"));

            Ok(())
        }
    }
}
