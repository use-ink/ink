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
        pub fn new(incrementer: Address) -> Self {
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
    use super::caller::{
        Caller,
        CallerRef,
    };
    use dyn_traits::Increment;
    use ink_e2e::ContractsBackend;
    use trait_incrementer::incrementer::{
        Incrementer,
        IncrementerRef,
    };

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    /// A test deploys and instantiates the `trait_incrementer::Incrementer` and
    /// `trait_incrementer_caller::Caller` contracts, where the `Caller` uses the account
    /// id of the `Incrementer` for instantiation.
    ///
    /// The test verifies that we can increment the value of the `Incrementer` contract
    /// through the `Caller` contract.
    #[ink_e2e::test]
    async fn e2e_cross_contract_calls<Client: E2EBackend>(
        mut client: Client,
    ) -> E2EResult<()> {
        let _ = client
            .upload("trait-incrementer", &ink_e2e::alice())
            .submit()
            .await
            .expect("uploading `trait-incrementer` failed")
            .code_hash;

        let _ = client
            .upload("trait-incrementer-caller", &ink_e2e::alice())
            .submit()
            .await
            .expect("uploading `trait-incrementer-caller` failed")
            .code_hash;

        let mut constructor = IncrementerRef::new();

        let incrementer = client
            .instantiate("trait-incrementer", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let incrementer_call = incrementer.call_builder::<Incrementer>();

        let mut constructor = CallerRef::new(incrementer.addr);

        let caller = client
            .instantiate(
                "trait-incrementer-caller",
                &ink_e2e::alice(),
                &mut constructor,
            )
            .submit()
            .await
            .expect("instantiate failed");
        let mut caller_call = caller.call_builder::<Caller>();

        // Check through the caller that the value of the incrementer is zero
        let get = caller_call.get();
        let value = client
            .call(&ink_e2e::alice(), &get)
            .dry_run()
            .await?
            .return_value();
        assert_eq!(value, 0);

        // Increment the value of the incrementer via the caller
        let inc = caller_call.inc();
        let _ = client
            .call(&ink_e2e::alice(), &inc)
            .submit()
            .await
            .expect("calling `inc` failed");

        // Ask the `trait-increment` about a value. It should be updated by the caller.
        // Also use `contract_ref!(Increment)` instead of `IncrementerRef`
        // to check that it also works with e2e testing.
        let get = incrementer_call.get();
        let value = client
            .call(&ink_e2e::alice(), &get)
            .dry_run()
            .await?
            .return_value();
        assert_eq!(value, 1);

        Ok(())
    }
}
