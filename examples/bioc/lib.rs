//! # Basic Input Output Contract (BIOC)
//!
//! This is a Basic Input Output Contract (BIOC), which is intented to
//! demonstrate rich i\o posibilties of ink! contracts, namely:
//!
//!   1. Use complicated nested input and ouput types.
//!      This is done through the real use case example of data structure
//!      needed to store a candle auction data.
//!   2. Make contract fail with `ContractTrapped`.
//!   3. Make contract fail with returning an Error.
//!   4. Perform debug printing from contract into node's log.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod bioc {
    use ink_prelude::{
        string::String,
        vec::Vec,
    };

    /// No storage is needed for this contract.
    #[ink(storage)]
    pub struct Bioc {}

    /// Struct for storing winning bids per bidding sample (a block).
    /// Vector index corresponds to sample number.
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Bids(Vec<Option<(AccountId, Balance)>>);

    /// Auction statuses.
    /// Logic inspired by
    /// [Parachain Auction](https://github.com/paritytech/polkadot/blob/master/runtime/common/src/traits.rs#L160)
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Status {
        /// An auction has not started yet.
        NotStarted,
        /// We are in the starting period of the auction, collecting initial bids.
        OpeningPeriod,
        /// We are in the ending period of the auction, where we are taking snapshots of the winning
        /// bids. Snapshots are taken currently on per-block basis, but this logic could be later evolve
        /// to take snapshots of on arbitrary length (in blocks)
        EndingPeriod(BlockNumber),
        /// Candle was blown
        Ended,
        /// We have completed the bidding process and are waiting for the Random Function to return some acceptable
        /// randomness to select the winner. The number represents how many blocks we have been waiting.
        RfDelay(BlockNumber),
    }

    /// Struct for storing auction data.
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Auction {
        /// Branded name of the auction event
        name: String,
        /// Some hash identifiyng the auction subject
        subject: Hash,
        /// Structure storing the bids being made
        bids: Bids,
        /// Auction terms encoded as:
        /// [start_block, opening_period, closing_period]
        terms: [BlockNumber; 3],
        /// Auction status
        status: Status,
        /// Candle auction can have no winner.
        /// If auction is finalized, that means that the winner is determined.
        finalized: bool,
    }

    /// Way to fail a contract execution.
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Failure {
        Revert,
        Panic,
    }

    impl Bioc {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Takes an auction data struct as input and returns it back.
        #[ink(message)]
        pub fn echo_auction(&mut self, auction: Auction) -> Auction {
            auction
        }

        /// Fails contract execution in the required way.
        #[ink(message)]
        pub fn revert_or_trap(&mut self, fail: Option<Failure>) -> Result<(), Failure> {
            match fail {
                Some(Failure::Revert) => Err(Failure::Revert),
                Some(Failure::Panic) => {
                    panic!("Trapping on user demand!")
                }
                None => Ok(()),
            }
        }

        /// Prints the specified string into node's debug log.
        #[ink(message)]
        pub fn debug_log(&mut self, str: String) {
            ink_env::debug_println!("debug_log: {:?}", str);
        }
    }
}
