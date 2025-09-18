#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod erc20 {
    use ink::{
        U256,
        storage::Mapping,
    };

    /// A simple ERC-20 contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Erc20 {
        /// Total token supply.
        total_supply: U256,
        /// Mapping from owner to number of owned token.
        balances: Mapping<Address, U256>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: Mapping<(Address, Address), U256>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<Address>,
        #[ink(topic)]
        to: Option<Address>,
        value: U256,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: Address,
        #[ink(topic)]
        spender: Address,
        value: U256,
    }

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
    }

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        pub fn new(total_supply: U256) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });
            Self {
                total_supply,
                balances,
                allowances: Default::default(),
            }
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> U256 {
            self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn balance_of(&self, owner: Address) -> U256 {
            self.balance_of_impl(&owner)
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `balance_of` since this
        /// works using references which are more efficient.
        #[inline]
        fn balance_of_impl(&self, owner: &Address) -> U256 {
            self.balances.get(owner).unwrap_or_default()
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        #[ink(message)]
        pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
            self.allowance_impl(&owner, &spender)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `allowance` since this
        /// works using references which are more efficient.
        #[inline]
        fn allowance_impl(&self, owner: &Address, spender: &Address) -> U256 {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message)]
        pub fn transfer(&mut self, to: Address, value: U256) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with
        /// `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        pub fn approve(&mut self, spender: Address, value: U256) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the account balance of `from`.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
            value: U256,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_from_to(&from, &to, value)?;
            // We checked that allowance >= value
            #[allow(clippy::arithmetic_side_effects)]
            self.allowances
                .insert((&from, &caller), &(allowance - value));
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: &Address,
            to: &Address,
            value: U256,
        ) -> Result<()> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance)
            }
            // We checked that from_balance >= value
            #[allow(clippy::arithmetic_side_effects)]
            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of_impl(to);
            self.balances
                .insert(to, &(to_balance.checked_add(value).unwrap()));
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            Ok(())
        }
    }

    #[cfg(test)]
    fn set_caller(sender: Address) {
        ink::env::test::set_caller(sender);
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink::primitives::{
            Clear,
            Hash,
        };

        fn assert_transfer_event(
            event: &ink::env::test::EmittedEvent,
            expected_from: Option<Address>,
            expected_to: Option<Address>,
            expected_value: U256,
        ) {
            let decoded_event =
                <Transfer as ink::scale::Decode>::decode(&mut &event.data[..])
                    .expect("encountered invalid contract event data buffer");
            let Transfer { from, to, value } = decoded_event;
            assert_eq!(from, expected_from, "encountered invalid Transfer.from");
            assert_eq!(to, expected_to, "encountered invalid Transfer.to");
            assert_eq!(value, expected_value, "encountered invalid Transfer.value");

            let mut expected_topics = Vec::new();
            expected_topics.push(
                ink::blake2x256!("Transfer(Option<Address>,Option<Address>,U256)").into(),
            );
            if let Some(from) = expected_from {
                expected_topics.push(encoded_into_hash(from));
            } else {
                expected_topics.push(Hash::CLEAR_HASH);
            }
            if let Some(to) = expected_to {
                expected_topics.push(encoded_into_hash(to));
            } else {
                expected_topics.push(Hash::CLEAR_HASH);
            }
            expected_topics.push(encoded_into_hash(value));

            let topics = event.topics.clone();
            for (n, (actual_topic, expected_topic)) in
                topics.iter().zip(expected_topics).enumerate()
            {
                let mut topic_hash = Hash::CLEAR_HASH;
                let len = actual_topic.len();
                topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

                assert_eq!(
                    topic_hash, expected_topic,
                    "encountered invalid topic at {n}"
                );
            }
        }

        /// The default constructor does its job.
        #[ink::test]
        fn new_works() {
            // Constructor works.
            set_caller(Address::from([0x01; 20]));
            let _erc20 = Erc20::new(100.into());

            // Transfer event triggered during initial construction.
            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(1, emitted_events.len());

            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(Address::from([0x01; 20])),
                100.into(),
            );
        }

        /// The total supply was applied.
        #[ink::test]
        fn total_supply_works() {
            // Constructor works.
            set_caller(Address::from([0x01; 20]));
            let erc20 = Erc20::new(100.into());
            // Transfer event triggered during initial construction.
            let emitted_events = ink::env::test::recorded_events();
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(Address::from([0x01; 20])),
                100.into(),
            );
            // Get the token total supply.
            assert_eq!(erc20.total_supply(), U256::from(100));
        }

        /// Get the actual balance of an account.
        #[ink::test]
        fn balance_of_works() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);

            // Constructor works
            let erc20 = Erc20::new(100.into());
            // Transfer event triggered during initial construction
            let emitted_events = ink::env::test::recorded_events();
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(accounts.alice),
                100.into(),
            );
            let accounts = ink::env::test::default_accounts();
            // Alice owns all the tokens on contract instantiation
            assert_eq!(erc20.balance_of(accounts.alice), U256::from(100));
            // Bob does not owns tokens
            assert_eq!(erc20.balance_of(accounts.bob), U256::zero());
        }

        #[ink::test]
        fn transfer_works() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);

            // Constructor works.
            let mut erc20 = Erc20::new(100.into());
            // Transfer event triggered during initial construction.
            assert_eq!(erc20.balance_of(accounts.bob), U256::zero());
            // Alice transfers 10 tokens to Bob.
            assert_eq!(erc20.transfer(accounts.bob, U256::from(10)), Ok(()));
            // Bob owns 10 tokens.
            assert_eq!(erc20.balance_of(accounts.bob), U256::from(10));

            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(emitted_events.len(), 2);
            // Check first transfer event related to ERC-20 instantiation.
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(accounts.alice),
                100.into(),
            );
            // Check the second transfer event relating to the actual transfer.
            assert_transfer_event(
                &emitted_events[1],
                Some(accounts.alice),
                Some(accounts.bob),
                10.into(),
            );
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            // Constructor works.
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);

            let initial_supply = 100.into();
            let mut erc20 = Erc20::new(initial_supply);

            assert_eq!(erc20.balance_of(accounts.bob), U256::zero());

            // Set the contract as callee and Bob as caller.
            let contract = ink::env::address();
            ink::env::test::set_callee(contract);
            set_caller(accounts.bob);

            // Bob fails to transfer 10 tokens to Eve.
            assert_eq!(
                erc20.transfer(accounts.eve, 10.into()),
                Err(Error::InsufficientBalance)
            );
            // Alice owns all the tokens.
            assert_eq!(erc20.balance_of(accounts.alice), U256::from(100));
            assert_eq!(erc20.balance_of(accounts.bob), U256::zero());
            assert_eq!(erc20.balance_of(accounts.eve), U256::zero());

            // Transfer event triggered during initial construction.
            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(emitted_events.len(), 1);
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(accounts.alice),
                100.into(),
            );
        }

        #[ink::test]
        fn transfer_from_works() {
            // Constructor works.
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);

            let mut erc20 = Erc20::new(100.into());

            // Bob fails to transfer tokens owned by Alice.
            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.eve, 10.into()),
                Err(Error::InsufficientAllowance)
            );
            // Alice approves Bob for token transfers on her behalf.
            assert_eq!(erc20.approve(accounts.bob, 10.into()), Ok(()));

            // The approve event takes place.
            assert_eq!(ink::env::test::recorded_events().len(), 2);

            // Set the contract as callee and Bob as caller.
            let contract = ink::env::address();
            ink::env::test::set_callee(contract);
            ink::env::test::set_caller(accounts.bob);

            // Bob transfers tokens from Alice to Eve.
            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.eve, 10.into()),
                Ok(())
            );
            // Eve owns tokens.
            assert_eq!(erc20.balance_of(accounts.eve), U256::from(10));

            // Check all transfer events that happened during the previous calls:
            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(emitted_events.len(), 3);
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(accounts.alice),
                100.into(),
            );
            // The second event `emitted_events[1]` is an Approve event that we skip
            // checking.
            assert_transfer_event(
                &emitted_events[2],
                Some(accounts.alice),
                Some(accounts.eve),
                10.into(),
            );
        }

        #[ink::test]
        fn allowance_must_not_change_on_failed_transfer() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            let mut erc20 = Erc20::new(100.into());

            // Alice approves Bob for token transfers on her behalf.
            let alice_balance = erc20.balance_of(accounts.alice);
            let initial_allowance = alice_balance + 2;
            assert_eq!(erc20.approve(accounts.bob, initial_allowance), Ok(()));

            // Get contract address.
            let callee = ink::env::address();
            ink::env::test::set_callee(callee);
            ink::env::test::set_caller(accounts.bob);

            // Bob tries to transfer tokens from Alice to Eve.
            let emitted_events_before = ink::env::test::recorded_events().len();
            assert_eq!(
                erc20.transfer_from(
                    accounts.alice,
                    accounts.eve,
                    alice_balance + U256::from(1)
                ),
                Err(Error::InsufficientBalance)
            );
            // Allowance must have stayed the same
            assert_eq!(
                erc20.allowance(accounts.alice, accounts.bob),
                initial_allowance
            );
            // No more events must have been emitted
            assert_eq!(
                emitted_events_before,
                ink::env::test::recorded_events().len()
            )
        }

        fn encoded_into_hash<T>(entity: T) -> Hash
        where
            T: ink::scale::Encode,
        {
            use ink::{
                env::hash::{
                    Blake2x256,
                    CryptoHash,
                    HashOutput,
                },
                primitives::Clear,
            };

            let mut result = Hash::CLEAR_HASH;
            let len_result = result.as_ref().len();
            let encoded = entity.encode();
            let len_encoded = encoded.len();
            if len_encoded <= len_result {
                result.as_mut()[..len_encoded].copy_from_slice(&encoded);
                return result
            }
            let mut hash_output =
                <<Blake2x256 as HashOutput>::Type as Default>::default();
            <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
            let copy_len = core::cmp::min(hash_output.len(), len_result);
            result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
            result
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_transfer<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let total_supply = U256::from(1_000_000_000);
            let mut constructor = Erc20Ref::new(total_supply);
            let erc20 = client
                .instantiate("erc20", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = erc20.call_builder::<Erc20>();

            // when
            let total_supply_msg = call_builder.total_supply();
            let total_supply_res = client
                .call(&ink_e2e::bob(), &total_supply_msg)
                .dry_run()
                .await?;

            let bob_account = ink_e2e::address::<ink::env::DefaultEnvironment>(
                ink_e2e::Sr25519Keyring::Bob,
            );
            let transfer_to_bob = U256::from(500_000_000);
            let transfer = call_builder.transfer(bob_account, transfer_to_bob);
            let _transfer_res = client
                .call(&ink_e2e::alice(), &transfer)
                .submit()
                .await
                .expect("transfer failed");

            let balance_of = call_builder.balance_of(bob_account);
            let balance_of_res = client
                .call(&ink_e2e::alice(), &balance_of)
                .dry_run()
                .await?;

            // then
            assert_eq!(
                total_supply,
                total_supply_res.return_value(),
                "total_supply"
            );
            assert_eq!(transfer_to_bob, balance_of_res.return_value(), "balance_of");

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_allowances<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let total_supply = U256::from(1_000_000_000);
            let mut constructor = Erc20Ref::new(total_supply);
            let erc20 = client
                .instantiate("erc20", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = erc20.call_builder::<Erc20>();

            // when

            let bob_account = ink_e2e::address::<ink::env::DefaultEnvironment>(
                ink_e2e::Sr25519Keyring::Bob,
            );
            let charlie_account = ink_e2e::address::<ink::env::DefaultEnvironment>(
                ink_e2e::Sr25519Keyring::Charlie,
            );

            let amount = U256::from(500_000_000);
            // tx
            let transfer_from =
                call_builder.transfer_from(bob_account, charlie_account, amount);
            let transfer_from_result = client
                .call(&ink_e2e::charlie(), &transfer_from)
                .submit()
                .await;

            assert!(
                transfer_from_result.is_err(),
                "unapproved transfer_from should fail"
            );

            // Bob approves Charlie to transfer up to amount on his behalf
            let approved_value = U256::from(1_000);
            let approve_call = call_builder.approve(charlie_account, approved_value);
            client
                .call(&ink_e2e::bob(), &approve_call)
                .submit()
                .await
                .expect("approve failed");

            // `transfer_from` the approved amount
            let transfer_from =
                call_builder.transfer_from(bob_account, charlie_account, approved_value);
            let transfer_from_result = client
                .call(&ink_e2e::charlie(), &transfer_from)
                .submit()
                .await;
            assert!(
                transfer_from_result.is_ok(),
                "approved transfer_from should succeed"
            );

            let balance_of = call_builder.balance_of(bob_account);
            let balance_of_res = client
                .call(&ink_e2e::alice(), &balance_of)
                .dry_run()
                .await?;

            // `transfer_from` again, this time exceeding the approved amount
            let transfer_from =
                call_builder.transfer_from(bob_account, charlie_account, 1.into());
            let transfer_from_result = client
                .call(&ink_e2e::charlie(), &transfer_from)
                .submit()
                .await;
            assert!(
                transfer_from_result.is_err(),
                "transfer_from exceeding the approved amount should fail"
            );

            assert_eq!(
                total_supply - approved_value,
                balance_of_res.return_value(),
                "balance_of"
            );

            Ok(())
        }
    }
}
