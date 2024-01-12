#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod delegator {
    use ink::{
        env::CallFlags,
        storage::Mapping,
    };

    use ink::{
        env::{
            call::{
                build_call,
                ExecutionInput,
                Selector,
            },
            DefaultEnvironment,
        },
        storage::traits::ManualKey,
    };

    #[ink(storage)]
    pub struct Delegator {
        addresses: Mapping<AccountId, i32, ManualKey<0x23>>,
        counter: i32,
    }

    impl Delegator {
        /// Creates a new delegator smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: i32) -> Self {
            let v = Mapping::new();
            Self {
                addresses: v,
                counter: init_value,
            }
        }

        /// Creates a new contract with default values.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        /// Increment the current value using delegate call.
        #[ink(message)]
        pub fn inc_delegate(&mut self, hash: Hash) {
            let selector = ink::selector_bytes!("inc");
            let _ = build_call::<DefaultEnvironment>()
                .delegate(hash)
                // We specify `set_tail_call(true)` to use the delegatee last memory frame
                // as the end of the execution cycle.
                // So any mutations to `Packed` types, made by delegatee,
                // will be flushed to storage.
                //
                // If we don't specify this flag.
                // The storage state before the delegate call will be flushed to storage instead.
                // See https://substrate.stackexchange.com/questions/3336/i-found-set-allow-reentry-may-have-some-problems/3352#3352
                .call_flags(CallFlags::default().set_tail_call(true))
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .try_invoke();
        }

        /// Adds entry to `addresses` using delegate call.
        /// Note that we don't need `set_tail_call(true)` flag
        /// because `Mapping` updates the storage instantly on-demand.
        #[ink(message)]
        pub fn add_entry_delegate(&mut self, hash: Hash) {
            let selector = ink::selector_bytes!("append_address_value");
            let _ = build_call::<DefaultEnvironment>()
                .delegate(hash)
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

            let mut constructor = DelegatorRef::new_default();
            let call_builder = client
                .instantiate("delegator", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder_call = call_builder.call::<Delegator>();

            let code_hash = client
                .upload("delegatee", &origin)
                .submit()
                .await
                .expect("upload `delegatee` failed")
                .code_hash;

            // when
            let call_delegate = call_builder_call.inc_delegate(code_hash);

            let result = client.call(&origin, &call_delegate).submit().await;
            assert!(result.is_ok(), "delegate call failed.");

            let result = client.call(&origin, &call_delegate).submit().await;
            assert!(result.is_ok(), "second delegate call failed.");

            // then
            let expected_value = 4;
            let call = call_builder.call::<Delegator>();

            let call_get = call.get_counter();
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

            // given
            let mut constructor = DelegatorRef::new(10);
            let call_builder = client
                .instantiate("delegator", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder_call = call_builder.call::<Delegator>();

            let code_hash = client
                .upload("delegatee", &origin)
                .submit()
                .await
                .expect("upload `delegatee` failed")
                .code_hash;

            // when
            let call_delegate = call_builder_call.add_entry_delegate(code_hash);
            let result = client.call(&origin, &call_delegate).submit().await;
            assert!(result.is_ok(), "delegate call failed.");

            // then

            // because we initialize the counter with `10` we expect this value be
            // assigned to Alice.
            let expected_value = 10;
            // Alice's address
            let address = AccountId::from(origin.public_key().to_account_id().0);

            let call_get_value = call_builder_call.get_value(address);
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
    }
}
