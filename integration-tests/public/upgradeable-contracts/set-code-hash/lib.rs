#![cfg_attr(not(feature = "std"), no_std, no_main)]

//! Demonstrates how to use [`set_code_hash`](https://docs.rs/ink_env/latest/ink_env/fn.set_code_hash.html)
//! to swap out the `code_hash` of an on-chain contract.
//!
//! We will swap the code of our `Incrementer` contract with that of the an `Incrementer`
//! found in the `updated_incrementer` folder.
//!
//! See the included End-to-End tests an example update workflow.

#[ink::contract]
pub mod incrementer {

    /// Track a counter in storage.
    ///
    /// # Note
    ///
    /// It is important to realize that after the call to `set_code_hash` the contract's
    /// storage remains the same.
    ///
    /// If you change the storage layout in your storage struct you may introduce
    /// undefined behavior to your contract!
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
            self.count = self.count.checked_add(1).unwrap();
            /*
            // todo
            ink::env::debug_println!(
                "The new count is {}, it was modified using the original contract code.",
                self.count
            );
            */
        }

        /// Returns the counter value which is stored in this contract's storage.
        #[ink(message)]
        pub fn get(&self) -> u32 {
            self.count
        }

        /// Modifies the code which is used to execute calls to this contract address
        /// (`AccountId`).
        ///
        /// We use this to upgrade the contract logic. We don't do any authorization here,
        /// any caller can execute this method.
        ///
        /// In a production contract you would do some authorization here!
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: ink::H256) {
            self.env().set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!("Failed to `set_code_hash` to {code_hash:?} due to {err:?}")
            });
            // todo
            // ink::env::debug_println!("Switched code hash to {:?}.", code_hash);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn set_code_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // Given
            let mut constructor = IncrementerRef::new();
            let contract = client
                .instantiate("incrementer", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Incrementer>();

            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::alice(), &get).dry_run().await?;
            assert!(matches!(get_res.return_value(), 0));

            let inc = call_builder.inc();
            let _inc_result = client
                .call(&ink_e2e::alice(), &inc)
                .submit()
                .await
                .expect("`inc` failed");

            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::alice(), &get).dry_run().await?;
            assert!(matches!(get_res.return_value(), 1));

            // When
            let new_code_hash = client
                .upload("updated_incrementer", &ink_e2e::alice())
                .submit()
                .await
                .expect("uploading `updated_incrementer` failed")
                .code_hash;

            let set_code = call_builder.set_code(new_code_hash);

            let _set_code_result = client
                .call(&ink_e2e::alice(), &set_code)
                .submit()
                .await
                .expect("`set_code` failed");

            // Then
            // Note that our contract's `AccountId` (so `contract_acc_id`) has stayed the
            // same between updates!
            let inc = call_builder.inc();

            let _inc_result = client
                .call(&ink_e2e::alice(), &inc)
                .submit()
                .await
                .expect("`inc` failed");

            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::alice(), &get).dry_run().await?;

            // Remember, we updated our incrementer contract to increment by `4`.
            assert!(matches!(get_res.return_value(), 5));

            Ok(())
        }
    }
}
