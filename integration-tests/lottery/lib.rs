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
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

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

    impl Default for Lottery {
        fn default() -> Self {
            Self {
                owner: [0u8; 32].into(),
                running: false,
                players: Vec::new(),
                entries: Mapping::default(),
            }
        }
    }

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
            self.entries.get(caller)
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
            let balance: Option<Balance> = self.entries.get(caller);

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
                value,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn pick_winner(&mut self) -> Result<()> {
            if self.players.is_empty() {
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
            assert!(lottery.random() > 0);
            assert!(!lottery.is_running());
        }
    }
}
