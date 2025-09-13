//! A smart contract which demonstrates behavior of the `self.env().transfer()` function.
//! It transfers some of it's balance to the caller.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod give_me {
    use ink::primitives::U256;

    /// No storage is needed for this simple contract.
    #[ink(storage)]
    pub struct GiveMe {}

    impl GiveMe {
        /// Creates a new instance of this contract.
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Self {}
        }

        /// Transfers `value` amount of tokens to the caller.
        ///
        /// # Errors
        ///
        /// - Panics in case the requested transfer exceeds the contract balance.
        /// - Panics in case the requested transfer would have brought this contract's
        ///   balance below the minimum balance (i.e. the chain's existential deposit).
        /// - Panics in case the transfer failed for another reason.
        #[ink(message)]
        pub fn give_me(&mut self, value: U256) {
            assert!(value <= self.env().balance(), "insufficient funds!");

            if self.env().transfer(self.env().caller(), value).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }
        }

        /// Asserts that the token amount sent as payment with this call
        /// is exactly `10`. This method will fail otherwise, and the
        /// transaction would then be reverted.
        ///
        /// # Note
        ///
        /// The method needs to be annotated with `payable`; only then it is
        /// allowed to receive value as part of the call.
        #[ink(message, payable, selector = 0xCAFEBABE)]
        pub fn was_it_ten(&mut self) {
            /*
            ink::env::debug_println!(
                "received payment: {}",
                self.env().transferred_value()
            );
            */
            assert!(
                self.env().transferred_value() == U256::from(10),
                "payment was not ten"
            );
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn transfer_works() {
            // given
            let contract_balance = 100.into();
            let accounts = default_accounts();
            let mut give_me = create_contract(contract_balance);

            // when
            set_sender(accounts.eve);
            set_balance(accounts.eve, 0.into());
            give_me.give_me(80.into());

            // then
            assert_eq!(get_balance(accounts.eve), 80.into());
        }

        #[ink::test]
        #[should_panic(expected = "insufficient funds!")]
        fn transfer_fails_insufficient_funds() {
            // given
            let contract_balance = 100.into();
            let accounts = default_accounts();
            let mut give_me = create_contract(contract_balance);

            // when
            set_sender(accounts.eve);
            give_me.give_me(120.into());

            // then
            // `give_me` must already have panicked here
        }

        #[ink::test]
        fn test_transferred_value() {
            use ink::codegen::Env;
            // given
            let accounts = default_accounts();
            let mut give_me = create_contract(100.into());
            let contract_account = give_me.env().address();

            // when
            // Push the new execution context which sets initial balances and
            // sets Eve as the caller
            set_balance(accounts.eve, 100.into());
            set_balance(contract_account, 0.into());
            set_sender(accounts.eve);

            // then
            // we use helper macro to emulate method invocation coming with payment,
            // and there must be no panic
            ink::env::pay_with_call!(give_me.was_it_ten(), 10.into());

            // and
            // balances should be changed properly
            let contract_new_balance = get_balance(contract_account);
            let caller_new_balance = get_balance(accounts.eve);

            assert_eq!(caller_new_balance, (100 - 10).into());
            assert_eq!(contract_new_balance, 10.into());
        }

        #[ink::test]
        #[should_panic(expected = "payment was not ten")]
        fn test_transferred_value_must_fail() {
            // given
            let accounts = default_accounts();
            let mut give_me = create_contract(100.into());

            // when
            // Push the new execution context which sets Eve as caller and
            // the `mock_transferred_value` as the value which the contract
            // will see as transferred to it.
            set_sender(accounts.eve);
            ink::env::test::set_value_transferred(13.into());

            // then
            give_me.was_it_ten();
        }

        /// Creates a new instance of `GiveMe` with `initial_balance`.
        ///
        /// Returns the `contract_instance`.
        fn create_contract(initial_balance: U256) -> GiveMe {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), initial_balance);
            GiveMe::new()
        }

        fn contract_id() -> Address {
            ink::env::test::callee()
        }

        fn set_sender(sender: Address) {
            ink::env::test::set_caller(sender);
        }

        fn default_accounts() -> ink::env::test::DefaultAccounts {
            ink::env::test::default_accounts()
        }

        fn set_balance(addr: Address, balance: U256) {
            ink::env::test::set_contract_balance(addr, balance)
        }

        fn get_balance(addr: Address) -> U256 {
            ink::env::test::get_contract_balance::<ink::env::DefaultEnvironment>(addr)
                .expect("Cannot get contract balance")
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::env::Environment;
        use ink_e2e::{
            ChainBackend,
            ContractsBackend,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_sending_value_to_give_me_must_fail<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = GiveMeRef::new();
            let contract = client
                .instantiate("contract_transfer", &ink_e2e::alice(), &mut constructor)
                .value(1_000_000_000)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<GiveMe>();

            // when
            let transfer = call_builder.give_me(120_000_000.into());

            let call_res = client
                .call(&ink_e2e::bob(), &transfer)
                .value(10_000_000)
                .submit()
                .await;

            // then
            assert!(call_res.is_err(), "call must have errored");
            /*
            // todo bug with wrong printing of message
            if let Err(ink_e2e::Error::CallDryRun(dry_run)) = call_res {
                assert!(dry_run.debug_message.contains("paid an unpayable message"))
            } else {
                panic!("Paying an unpayable message should fail")
            }
            */
            Ok(())
        }

        #[ink_e2e::test(backend(runtime_only))]
        async fn e2e_contract_must_transfer_value_to_sender<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = GiveMeRef::new();
            let contract = client
                .instantiate("contract_transfer", &ink_e2e::bob(), &mut constructor)
                // todo convert the argument type to U256
                .value(1_337_000_000)
                .submit()
                .await
                .expect("instantiate failed");
            let contract_addr = contract.addr;

            assert_eq!(
                contract.trace.clone().unwrap().value,
                Some(ink::env::DefaultEnvironment::native_to_eth(1_337_000_000))
            );
            let mut call_builder = contract.call_builder::<GiveMe>();

            let balance_before: Balance = client
                .free_balance(contract.account_id)
                .await
                .expect("getting balance failed");

            // when
            let transfer = call_builder.give_me(U256::from(120_000_000_0));

            let call_res = client
                .call(&ink_e2e::eve(), &transfer)
                .submit()
                .await
                .expect("call failed");

            // then
            let outgoing_trace = &call_res.trace.unwrap().calls[0];
            assert_eq!(outgoing_trace.value, Some(U256::from(120_000_000_0)));
            assert_eq!(outgoing_trace.from, contract_addr);
            assert_eq!(
                outgoing_trace.to,
                ink_e2e::address_from_keypair::<AccountId>(&ink_e2e::eve())
            );

            let balance_after: Balance = client
                .free_balance(contract.account_id)
                .await
                .expect("getting balance failed");
            assert_eq!(balance_before - balance_after, 12);

            Ok(())
        }
    }
}
