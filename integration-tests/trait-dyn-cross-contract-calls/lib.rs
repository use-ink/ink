//! This crate contains the `Caller` contract with no functionality except forwarding
//! all calls to the `trait_incrementer::Incrementer` contract.
//!
//! The `Caller` doesn't use the `trait_incrementer::IncrementerRef`. Instead,
//! all interactions with the `Incrementer` is done through the wrapper from
//! `ink::contract_ref!` and the trait `dyn_traits::Increment`.
#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod caller {
    use dyn_traits::Increment;

    /// The caller of the incrementer smart contract.
    #[ink(storage)]
    pub struct Caller {
        /// Here we accept a type which implements the `Incrementer` ink! trait.
        incrementer: ink::contract_ref!(Increment),
    }

    impl Caller {
        /// Creates a new caller smart contract around the `incrementer` account id.
        #[ink(constructor)]
        pub fn new(incrementer: AccountId) -> Self {
            Self {
                incrementer: incrementer.into(),
            }
        }
    }

    impl Increment for Caller {
        #[ink(message)]
        fn inc(&mut self) {
            self.incrementer.inc()
        }

        #[ink(message)]
        fn get(&self) -> u64 {
            self.incrementer.get()
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::caller::CallerRef;
    use dyn_traits::Increment;
    use ink::{
        contract_ref,
        env::DefaultEnvironment,
    };
    use ink_e2e::build_message;
    use trait_incrementer::incrementer::IncrementerRef;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    /// A test deploys and instantiates the `trait_incrementer::Incrementer` and
    /// `trait_incrementer_caller::Caller` contracts, where the `Caller` uses the account
    /// id of the `Incrementer` for instantiation.
    ///
    /// The test verifies that we can increment the value of the `Incrementer` contract
    /// through the `Caller` contract.
    #[ink_e2e::test(additional_contracts = "contracts/incrementer/Cargo.toml")]
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

        let constructor = IncrementerRef::new();

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

        // Check through the caller that the value of the incrementer is zero
        let get = build_message::<CallerRef>(caller_account_id.clone())
            .call(|contract| contract.get());
        let value = client
            .call_dry_run(&ink_e2e::alice(), &get, 0, None)
            .await
            .return_value();
        assert_eq!(value, 0);

        // Increment the value of the incrementer via the caller
        let inc = build_message::<CallerRef>(caller_account_id.clone())
            .call(|contract| contract.inc());
        let _ = client
            .call(&ink_e2e::alice(), inc, 0, None)
            .await
            .expect("calling `inc` failed");

        // Ask the `trait-increment` about a value. It should be updated by the caller.
        // Also use `contract_ref!(Increment)` instead of `IncrementerRef`
        // to check that it also works with e2e testing.
        let get = build_message::<contract_ref!(Increment, DefaultEnvironment)>(
            incrementer_account_id.clone(),
        )
        .call(|contract| contract.get());
        let value = client
            .call_dry_run(&ink_e2e::alice(), &get, 0, None)
            .await
            .return_value();
        assert_eq!(value, 1);

        Ok(())
    }
}
