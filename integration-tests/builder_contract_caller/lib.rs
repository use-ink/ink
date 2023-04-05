//! The integration test is the same as `../basic_contract_caller`
//! but it uses the [`ink::env::call::CallBuilder`] to do cross contract calls.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod builder_contract_caller {
    use ink::env::call_builder;
    /// We import the generated `ContractRef` of our other contract.
    ///
    /// Note that the other contract must have re-exported it (`pub use
    /// OtherContractRef`) for us to have access to it.
    use other_contract::OtherContractRef;

    #[ink(storage)]
    pub struct BuilderContractCaller {
        /// We specify that our contract will store a reference to the `OtherContract`.
        other_contract: OtherContractRef,
    }

    impl BuilderContractCaller {
        /// In order to use the `OtherContract` we first need to **instantiate** it.
        ///
        /// To do this we will use the uploaded `code_hash` of `OtherContract`.
        #[ink(constructor)]
        pub fn new(other_contract_code_hash: Hash) -> Self {
            let other_contract = OtherContractRef::new(true)
                .code_hash(other_contract_code_hash)
                .endowment(0)
                .salt_bytes([0xDE, 0xAD, 0xBE, 0xEF])
                .instantiate();

            Self { other_contract }
        }

        /// Return the value of the `OtherContract`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            use other_contract::Trait;
            call_builder!(self.other_contract.get()).invoke()
        }

        /// Using the `ContractRef` we can call all the messages of the `OtherContract` as
        /// if they were normal Rust methods (because at the end of the day, they
        /// are!).
        #[ink(message)]
        pub fn flip_and_get(&mut self) -> bool {
            call_builder!(self.other_contract.flip()).invoke();
            call_builder!(other_contract::Trait::get(&self.other_contract)).invoke()
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::builder_contract_caller::BuilderContractCallerRef;
    use ink_e2e::build_message;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    /// A test deploys and instantiates the
    /// `builder_contract_caller::BuilderContractCaller` contract.
    ///
    /// The test verifies that we can call `BuilderContractCaller::flip_and_get`.
    #[ink_e2e::test(additional_contracts = "other_contract/Cargo.toml")]
    async fn e2e_cross_contract_calls(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let other_code_hash = client
            .upload("other-contract", &ink_e2e::alice(), None)
            .await
            .expect("uploading `other-contract` failed")
            .code_hash;

        let _ = client
            .upload("builder-contract-caller", &ink_e2e::alice(), None)
            .await
            .expect("uploading `builder-contract-caller` failed")
            .code_hash;

        let constructor = BuilderContractCallerRef::new(other_code_hash);

        let caller_account_id = client
            .instantiate(
                "builder-contract-caller",
                &ink_e2e::alice(),
                constructor,
                0,
                None,
            )
            .await
            .expect("instantiate failed")
            .account_id;

        // Check that the `get` return `true`(default value).
        let get = build_message::<BuilderContractCallerRef>(caller_account_id.clone())
            .call(|contract| contract.get());
        let value = client
            .call_dry_run(&ink_e2e::alice(), &get, 0, None)
            .await
            .return_value();
        assert_eq!(value, true);

        // Flip the value
        let flip_and_get =
            build_message::<BuilderContractCallerRef>(caller_account_id.clone())
                .call(|contract| contract.flip_and_get());
        let _ = client
            .call(&ink_e2e::alice(), flip_and_get, 0, None)
            .await
            .expect("calling `flip_and_get` failed");

        // The value should be updated
        let get = build_message::<BuilderContractCallerRef>(caller_account_id.clone())
            .call(|contract| contract.get());
        let value = client
            .call_dry_run(&ink_e2e::alice(), &get, 0, None)
            .await
            .return_value();
        assert_eq!(value, false);

        // The dry run for `flip_and_get` should return `true` again.
        let flip_and_get =
            build_message::<BuilderContractCallerRef>(caller_account_id.clone())
                .call(|contract| contract.flip_and_get());
        let value = client
            .call_dry_run(&ink_e2e::alice(), &flip_and_get, 0, None)
            .await
            .return_value();
        assert_eq!(value, true);

        Ok(())
    }
}
