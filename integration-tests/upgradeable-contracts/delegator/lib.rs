#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod delegator {
    use ink::{
        env::{
            call::{
                build_call,
                ExecutionInput,
                Selector,
            },
            CallFlags,
            DefaultEnvironment,
        },
        storage::{
            traits::ManualKey,
            Lazy,
            Mapping,
        },
    };

    #[ink(storage)]
    pub struct Delegator {
        addresses: Mapping<AccountId, i32, ManualKey<0x23>>,
        counter: i32,
        delegate_to: Lazy<Hash>,
    }

    impl Delegator {
        /// Creates a new delegator smart contract with an initial value, and the hash of
        /// the contract code to delegate to.
        ///
        /// Additionally, this code hash will be locked to prevent its deletion, since
        /// this contract depends on it.
        #[ink(constructor)]
        pub fn new(init_value: i32, hash: Hash) -> Self {
            let v = Mapping::new();

            // Initialize the hash of the contract to delegate to.
            // Adds a delegate dependency lock, ensuring that the delegated to code cannot
            // be removed.
            let mut delegate_to = Lazy::new();
            delegate_to.set(&hash);
            Self::env().lock_delegate_dependency(&hash);

            Self {
                addresses: v,
                counter: init_value,
                delegate_to,
            }
        }

        /// Update the hash of the contract to delegate to.
        /// - Unlocks the old delegate dependency, releasing the deposit and allowing old
        ///   delegated to code to be removed.
        /// - Adds a new delegate dependency lock, ensuring that the new delegated to code
        ///   cannot be removed.
        #[ink(message)]
        pub fn update_delegate_to(&mut self, hash: Hash) {
            if let Some(old_hash) = self.delegate_to.get() {
                self.env().unlock_delegate_dependency(&old_hash)
            }
            self.env().lock_delegate_dependency(&hash);
            self.delegate_to.set(&hash);
        }

        /// Increment the current value using delegate call.
        #[ink(message)]
        pub fn inc_delegate(&mut self) {
            let selector = ink::selector_bytes!("inc");
            let _ = build_call::<DefaultEnvironment>()
                .delegate(self.delegate_to())
                // We specify `CallFlags::TAIL_CALL` to use the delegatee last memory frame
                // as the end of the execution cycle.
                // So any mutations to `Packed` types, made by delegatee,
                // will be flushed to storage.
                //
                // If we don't specify this flag.
                // The storage state before the delegate call will be flushed to storage instead.
                // See https://substrate.stackexchange.com/questions/3336/i-found-set-allow-reentry-may-have-some-problems/3352#3352
                .call_flags(CallFlags::TAIL_CALL)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .try_invoke();
        }

        /// Adds entry to `addresses` using delegate call.
        /// Note that we don't need `CallFlags::TAIL_CALL` flag
        /// because `Mapping` updates the storage instantly on-demand.
        #[ink(message)]
        pub fn add_entry_delegate(&mut self) {
            let selector = ink::selector_bytes!("append_address_value");
            let _ = build_call::<DefaultEnvironment>()
                .delegate(self.delegate_to())
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .try_invoke();
        }

        /// Returns the current value of the counter.
        #[ink(message)]
        pub fn get_counter(&self) -> i32 {
            self.counter
        }

        /// Returns the current value of the address.
        #[ink(message)]
        pub fn get_value(&self, address: AccountId) -> (AccountId, Option<i32>) {
            (self.env().caller(), self.addresses.get(address))
        }

        fn delegate_to(&self) -> Hash {
            self.delegate_to
                .get()
                .expect("delegate_to always has a value")
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{
            ChainBackend,
            ContractsBackend,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_counter_mutated<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let origin = client
                .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
                .await;

            let code_hash = client
                .upload("delegatee", &origin)
                .submit()
                .await
                .expect("upload `delegatee` failed")
                .code_hash;

            let mut constructor = DelegatorRef::new(0, code_hash);
            let contract = client
                .instantiate("delegator", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Delegator>();

            // when
            let call_delegate = call_builder.inc_delegate();

            let result = client.call(&origin, &call_delegate).submit().await;
            assert!(result.is_ok(), "delegate call failed.");

            let result = client.call(&origin, &call_delegate).submit().await;
            assert!(result.is_ok(), "second delegate call failed.");

            // then
            let expected_value = 4;
            let call_builder = contract.call_builder::<Delegator>();

            let call_get = call_builder.get_counter();
            let call_get_result = client
                .call(&origin, &call_get)
                .dry_run()
                .await?
                .return_value();

            // This fails
            assert_eq!(
                call_get_result, expected_value,
                "Decoded an unexpected value from the call."
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_mapping_mutated<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
                .await;

            let code_hash = client
                .upload("delegatee", &origin)
                .submit()
                .await
                .expect("upload `delegatee` failed")
                .code_hash;

            // given
            let mut constructor = DelegatorRef::new(10, code_hash);
            let contract = client
                .instantiate("delegator", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Delegator>();

            // when
            let call_delegate = call_builder.add_entry_delegate();
            let result = client.call(&origin, &call_delegate).submit().await;
            assert!(result.is_ok(), "delegate call failed.");

            // then

            // because we initialize the counter with `10` we expect this value be
            // assigned to Alice.
            let expected_value = 10;
            // Alice's address
            let address = AccountId::from(origin.public_key().to_account_id().0);

            let call_get_value = call_builder.get_value(address);
            let call_get_result = client
                .call(&origin, &call_get_value)
                .submit()
                .await
                .unwrap()
                .return_value();

            assert_eq!(
                call_get_result,
                (address, Some(expected_value)),
                "Decoded an unexpected value from the call."
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn update_delegate<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let origin = client
                .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
                .await;

            let code_hash = client
                .upload("delegatee", &origin)
                .submit()
                .await
                .expect("upload `delegatee` failed")
                .code_hash;

            let code_hash2 = client
                .upload("delegatee2", &origin)
                .submit()
                .await
                .expect("upload `delegatee2` failed")
                .code_hash;

            let mut constructor = DelegatorRef::new(10, code_hash);
            let contract = client
                .instantiate("delegator", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Delegator>();

            // when
            let call_delegate = call_builder.update_delegate_to(code_hash2);
            let result = client.call(&origin, &call_delegate).submit().await;
            assert!(result.is_ok(), "update_delegate_to failed.");

            // then

            // remove the original delegatee code.
            // should succeed because the delegate dependency has been removed.
            let original_code_removed =
                client.remove_code(&origin, code_hash).submit().await;
            assert!(original_code_removed.is_ok());

            // attempt to remove the new delegatee code.
            // should fail because of the delegate dependency.
            let new_code_removed = client.remove_code(&origin, code_hash2).submit().await;
            assert!(new_code_removed.is_err());

            Ok(())
        }
    }
}
