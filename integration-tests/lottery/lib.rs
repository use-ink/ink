//! # Lottery Contract
//!
//! This contract implements a simple lottery in ink!.
//! It is used to demonstrate the capabilities of ink!.
//!
//! ## Overview
//!
//! The lottery contract allows players to enter the lottery by sending a value.
//! The lottery can be started and stopped by the owner.
//! Anyone can pick a winner and the pot is transferred to the winner.
//! Contract uses pseudo random number generator to pick a winner. Please dont use this in production.
//!

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod lottery {
    use ink::env::hash::Keccak256;
    use ink::storage::{Mapping};
    use ink::prelude::vec::Vec;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Lottery {
        owner: AccountId,
        running: bool,
        players: Vec<AccountId>,
        entries: Mapping<AccountId, Balance>,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if Lottery is not running.
        LotteryNotRunning,
        /// Returned if caller is not owner while required to.
        CallerNotOwner,
        /// Returned if no value was sent.
        NoValueSent,
        /// Returned if transfer failed.
        ErrTransfer,
        /// Returned if the player is already in the lottery.
        PlayerAlreadyInLottery,
        /// Returned if there are no entries.
        NoEntries,
    }

    #[ink(event)]
    pub struct Entered {
        /// The player who entered.
        player: AccountId,
        /// The value sent.
        value: Balance,
    }

    #[ink(event)]
    pub struct Won {
        /// The winner.
        winner: AccountId,
        /// The winning amount.
        amount: Balance,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Lottery {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                owner: Self::env().caller(),
                running: false,
                players: Vec::new(),
                entries: Mapping::default(),
            }
        }

        /// Returns the current owner of the lottery
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        pub fn pot(&self) -> Balance {
            self.env().balance()
        }

        /// Returns the current state of the lottery
        #[ink(message)]
        pub fn is_running(&self) -> bool {
            self.running
        }

        /// Returns the list of players
        #[ink(message)]
        pub fn get_players(&self) -> Vec<AccountId> {
            self.players.clone()
        }

        /// Retrieve the balance of the account.
        #[ink(message)]
        pub fn get_balance(&self, caller: AccountId) -> Option<Balance> {
            self.entries.get(&caller)
        }

        /// Generates a seed based on the list of players and the block number and timestamp
        fn seed(&self) -> u64 {
            let hash = self.env().hash_encoded::<Keccak256, _>(&self.players);
            let num = u64::from_be_bytes(hash[0..8].try_into().unwrap());
            let timestamp = self.env().block_timestamp();
            let block_number = self.env().block_number() as u64;

            num ^ timestamp ^ block_number
        }

        /// Pseudo random number generator
        fn random(&self) -> u64 {
            let mut x = self.seed();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;

            x
        }

        /// Allows a player to enter the lottery by sending a value
        #[ink(message, payable)]
        pub fn enter(&mut self) -> Result<()> {
            if !self.running {
                return Err(Error::LotteryNotRunning);
            }
            let caller = self.env().caller();
            let balance: Option<Balance> = self.entries.get(&caller);

            if balance.is_some() {
                return Err(Error::PlayerAlreadyInLottery);
            }

            let value: Balance = self.env().transferred_value();
            if value < 1 {
                return Err(Error::NoValueSent);
            }

            self.players.push(caller);
            self.entries.insert(caller, &value);

            self.env().emit_event(Entered {
                player: caller,
                value: value,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn pick_winner(&mut self) -> Result<()> {
            if self.players.len() == 0 {
                return Err(Error::NoEntries);
            }
            let winner_index = self.random() % self.players.len() as u64;
            let winner = self.players[winner_index as usize];
            let amount: Balance = self.env().balance();

            if self.env().transfer(winner, amount).is_err() {
                return Err(Error::ErrTransfer);
            }

            for player in self.players.iter() {
                self.entries.remove(player);
            }

            self.players = Vec::new();

            self.env().emit_event(Won { winner, amount });

            Ok(())
        }

        #[ink(message)]
        pub fn start_lottery(&mut self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::CallerNotOwner);
            }
            self.running = true;

            Ok(())
        }

        #[ink(message)]
        pub fn stop_lottery(&mut self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::CallerNotOwner);
            }
            self.running = false;

            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let lottery = Lottery::new();
            assert_eq!(lottery.random() > 0, true);
        }
    }

    /// end-to-end (E2E) or integration tests for lottery.
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        fn get_alice_and_bob() -> (AccountId, AccountId) {
            let alice = ink_e2e::alice::<ink_e2e::PolkadotConfig>();
            let alice_account_id_32 = alice.account_id();
            let alice_account_id = AccountId::try_from(alice_account_id_32.as_ref()).unwrap();

            let bob = ink_e2e::bob::<ink_e2e::PolkadotConfig>();
            let bob_account_id_32 = bob.account_id();
            let bob_account_id = AccountId::try_from(bob_account_id_32.as_ref()).unwrap();

            (alice_account_id, bob_account_id)
        }

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn init_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = LotteryRef::new();

            // When
            let contract_account_id = client
                .instantiate("lottery", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let owner = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.owner());
            let owner_result = client.call_dry_run(&ink_e2e::alice(), &owner, 0, None).await;
            let (_alice_account_id, _bob_account_id) = get_alice_and_bob();

            assert!(matches!(owner_result.return_value(), _alice_account_id));

            Ok(())
        }

        /// We test that we can run lottery.
        #[ink_e2e::test]
        async fn lottery_flow(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = LotteryRef::new();
            let contract_account_id = client
                .instantiate("lottery", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let owner = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.owner());
            let owner_result = client.call_dry_run(&ink_e2e::bob(), &owner, 0, None).await;
            let (_alice_account_id, _bob_account_id) = get_alice_and_bob();

            assert!(matches!(owner_result.return_value(), _bob_account_id));

            // When
            let start_lottery = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.start_lottery());
            let _start_lottery_result = client
                .call(&ink_e2e::bob(), start_lottery, 0, None)
                .await
                .expect("start_lottery failed");

            // Then
            let running = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.is_running());
            let running_result = client.call_dry_run(&ink_e2e::bob(), &running, 0, None).await;
            assert!(matches!(running_result.return_value(), true));

            // When
            let stop_lottery = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.stop_lottery());
            let _stop_lottery_result = client
                .call(&ink_e2e::bob(), stop_lottery, 0, None)
                .await
                .expect("stop_lottery failed");

            // Then
            let running_after_stop = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.is_running());
            let running_after_stop_result = client.call_dry_run(&ink_e2e::bob(), &running_after_stop, 0, None).await;
            assert!(matches!(running_after_stop_result.return_value(), false));


            // Start Lottery again
            let start_again = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.start_lottery());
            client
                .call(&ink_e2e::bob(), start_again, 0, None)
                .await
                .expect("start_lottery failed");

            // Enter
            let enter = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.enter());
            let _enter_result = client
                .call(&ink_e2e::alice(), enter, 1000, None)
                .await
                .expect("enter lottery failed");

            // Check pot
            let pot = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.pot());
            let pot_result = client.call_dry_run(&ink_e2e::alice(), &pot, 0, None).await;
            assert!(&pot_result.return_value() > &1u128);

            // Second enter
            let enter_again = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.enter());
            client
                .call(&ink_e2e::bob(), enter_again, 1000, None)
                .await
                .expect("enter lottery failed");

            let pot2 = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.pot());
            let pot_result2 = client.call_dry_run(&ink_e2e::alice(), &pot2, 0, None).await;
            assert!(&pot_result2.return_value() > &1u128);

            Ok(())
        }
    }
}
