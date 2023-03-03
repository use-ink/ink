#![cfg_attr(not(feature = "std"), no_std)]

//! Demonstrates how to use [`set_code_hash`](https://docs.rs/ink_env/latest/ink_env/fn.set_code_hash.html)
//! to swap out the `code_hash` of an on-chain contract.
//!
//! We will swap the code of our `Incrementer` contract with that of the an `Incrementer` found in
//! the `updated_incrementer` folder.
//!
//! See the included End-to-End tests an example update workflow.

#[ink::contract]
pub mod incrementer {

    /// Track a counter in storage.
    ///
    /// # Note
    ///
    /// Is is important to realize that after the call to `set_code_hash` the contract's storage
    /// remains the same.
    ///
    /// If you change the storage layout in your storage struct you may introduce undefined
    /// behavior to your contract!
    #[ink(storage)]
    #[derive(Default)]
    pub struct Incrementer {
        count: u32,
    }

    impl Incrementer {
        /// Creates a new counter smart contract initialized with the given base value.
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Increments the counter value which is stored in the contract's storage.
        #[ink(message)]
        pub fn inc(&mut self) {
            self.count += 1;
            ink::env::debug_println!(
                "The new count is {}, it was modified using the original contract code.",
                self.count
            );
        }

        /// Returns the counter value which is stored in this contract's storage.
        #[ink(message)]
        pub fn get(&self) -> u32 {
            self.count
        }

        /// Modifies the code which is used to execute calls to this contract address (`AccountId`).
        ///
        /// We use this to upgrade the contract logic. We don't do any authorization here, any caller
        /// can execute this method.
        ///
        /// In a production contract you would do some authorization here!
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: [u8; 32]) {
            ink::env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!("Failed to `set_code_hash` to {code_hash:?} due to {err:?}")
            });
            ink::env::debug_println!("Switched code hash to {:?}.", code_hash);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(additional_contracts = "./updated_incrementer/Cargo.toml")]
        async fn set_code_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = IncrementerRef::new();
            let contract_acc_id = client
                .instantiate("incrementer", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<IncrementerRef>(contract_acc_id.clone())
                .call(|incrementer| incrementer.get());
            let get_res = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_res.return_value(), 0));

            let inc = build_message::<IncrementerRef>(contract_acc_id.clone())
                .call(|incrementer| incrementer.inc());
            let _inc_result = client
                .call(&ink_e2e::alice(), inc, 0, None)
                .await
                .expect("`inc` failed");

            let get = build_message::<IncrementerRef>(contract_acc_id.clone())
                .call(|incrementer| incrementer.get());
            let get_res = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_res.return_value(), 1));

            // When
            let new_code_hash = client
                .upload("updated_incrementer", &ink_e2e::alice(), None)
                .await
                .expect("uploading `updated_incrementer` failed")
                .code_hash;

            let new_code_hash = new_code_hash.as_ref().try_into().unwrap();
            let set_code = build_message::<IncrementerRef>(contract_acc_id.clone())
                .call(|incrementer| incrementer.set_code(new_code_hash));

            let _set_code_result = client
                .call(&ink_e2e::alice(), set_code, 0, None)
                .await
                .expect("`set_code` failed");

            // Then
            // Note that our contract's `AccountId` (so `contract_acc_id`) has stayed the same
            // between updates!
            let inc = build_message::<IncrementerRef>(contract_acc_id.clone())
                .call(|incrementer| incrementer.inc());

            let _inc_result = client
                .call(&ink_e2e::alice(), inc, 0, None)
                .await
                .expect("`inc` failed");

            let get = build_message::<IncrementerRef>(contract_acc_id.clone())
                .call(|incrementer| incrementer.get());
            let get_res = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;

            // Remember, we updated our incrementer contract to increment by `4`.
            assert!(matches!(get_res.return_value(), 5));

            Ok(())
        }
    }
}
