#![cfg_attr(not(feature = "std"), no_std)]

/// We can't run E2E test without the contract. So add `Dummy` contract.
#[ink::contract]
pub mod dummy {
    #[ink(storage)]
    #[derive(Default)]
    pub struct Dummy {}

    impl Dummy {
        /// Dummy constructor.
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Dummy method.
        #[ink(message)]
        pub fn workaround(&self) {}
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use dyn_traits::Increment;
    use ink::contract_ref;
    use ink_e2e::build_message;
    use trait_incrementer::incrementer::IncrementerRef;
    use trait_incrementer_caller::caller::CallerRef;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test(
        additional_contracts = "contracts/incrementer/Cargo.toml contracts/incrementer-caller/Cargo.toml"
    )]
    async fn e2e_cross_contract_calls(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let _ = client
            .upload("trait-incrementer", &ink_e2e::alice(), None)
            .await
            .expect("uploading `trait-incrementer` failed")
            .code_hash;

        let _ = client
            .upload("trait-incrementer-caller", &ink_e2e::alice(), None)
            .await
            .expect("uploading `trait-incrementer-caller` failed")
            .code_hash;

        let constructor = IncrementerRef::new(
            0, // initial value
        );

        let incrementer_account_id = client
            .instantiate("trait-incrementer", &ink_e2e::alice(), constructor, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;

        let constructor = CallerRef::new(incrementer_account_id.clone());

        let caller_account_id = client
            .instantiate(
                "trait-incrementer-caller",
                &ink_e2e::alice(),
                constructor,
                0,
                None,
            )
            .await
            .expect("instantiate failed")
            .account_id;

        // Check throw the caller that the value of the incrementer is zero
        let get = build_message::<CallerRef>(caller_account_id.clone())
            .call(|contract| contract.get());
        let value = client
            .call_dry_run(&ink_e2e::bob(), &get, 0, None)
            .await
            .return_value();
        assert_eq!(value, 0);

        // Increment the value of the incrementer via the caller
        let inc = build_message::<CallerRef>(caller_account_id.clone())
            .call(|contract| contract.inc());
        let _ = client
            .call(&ink_e2e::bob(), inc, 0, None)
            .await
            .expect("calling `inc` failed");

        // Ask the `trait-increment` about a value. It should be updated by the caller.
        // Also use `contract_ref!(Increment)` instead of `IncrementerRef`
        // to check that it also works with e2e testing.
        let get =
            build_message::<contract_ref!(Increment)>(incrementer_account_id.clone())
                .call(|contract| contract.get());
        let value = client
            .call_dry_run(&ink_e2e::bob(), &get, 0, None)
            .await
            .return_value();
        assert_eq!(value, 1);

        Ok(())
    }
}
