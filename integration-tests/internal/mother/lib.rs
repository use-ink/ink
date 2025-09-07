//! # The Mother of All Contracts
//!
//! This contract is intended to make use of all features that are observable
//! by off-chain tooling (for example user interfaces).
//! It doesn't do anything useful beyond serving off-chain tooling developers
//! with a contract to test their software against.
//!
//! Currently, this includes the following:
//!
//!   1. Use complex nested input and output types. This is done with the use case of a
//!      data structure needed to store data of a candle auction.
//!   2. Make contract fail with `ContractTrapped`.
//!   3. Make contract fail with returning an `Error`.
//!   4. Perform debug printing from contract into the node's log.
//!   5. Use complex types in storage.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod mother {
    use ink::prelude::{
        format,
        string::{
            String,
            ToString,
        },
        vec::Vec,
    };

    use ink::storage::{
        Mapping,
        StorageVec,
    };

    /// Struct for storing winning bids per bidding sample (a block).
    /// Vector index corresponds to sample number.
    /// Wrapping vector, just added for testing UI components.
    #[derive(Default, PartialEq, Eq, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Bids(Vec<Vec<Option<(Address, Balance)>>>);

    /// Auction outline.
    #[derive(PartialEq, Eq, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Outline {
        NoWinner,
        WinnerDetected,
        PayoutCompleted,
    }

    /// Auction statuses.
    /// Logic inspired by
    /// [Parachain Auction](https://github.com/paritytech/polkadot/blob/master/runtime/common/src/traits.rs#L160)
    #[derive(PartialEq, Eq, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Status {
        /// An auction has not started yet.
        NotStarted,
        /// We are in the starting period of the auction, collecting initial bids.
        OpeningPeriod,
        /// We are in the ending period of the auction, where we are taking snapshots of
        /// the winning bids. Snapshots are taken currently on per-block basis,
        /// but this logic could be later evolve to take snapshots of on
        /// arbitrary length (in blocks).
        EndingPeriod(BlockNumber),
        /// Candle was blown.
        Ended(Outline),
        /// We have completed the bidding process and are waiting for the Random Function
        /// to return some acceptable randomness to select the winner. The number
        /// represents how many blocks we have been waiting.
        RfDelay(BlockNumber),
    }

    /// Struct for storing auction data.
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Auction {
        /// Branded name of the auction event.
        name: String,
        /// Some hash identifying the auction subject.
        subject: Hash,
        /// Structure storing the bids being made.
        bids: Bids,
        /// Auction terms encoded as:
        /// `[start_block, opening_period, closing_period]`
        terms: [BlockNumber; 3],
        /// Auction status.
        status: Status,
        /// Candle auction can have no winner.
        /// If auction is finalized, that means that the winner is determined.
        finalized: bool,
        /// Just a vector for the UI tests.
        vector: Vec<u8>,
    }

    impl Default for Auction {
        fn default() -> Auction {
            Auction {
                name: String::default(),
                subject: Hash::default(),
                bids: Bids::default(),
                terms: <[BlockNumber; 3]>::default(),
                status: Status::OpeningPeriod,
                finalized: false,
                vector: <Vec<u8>>::default(),
            }
        }
    }

    /// Way to fail a contract execution.
    #[derive(Debug, Eq, PartialEq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Failure {
        Revert(String),
        Panic,
    }

    /// Event emitted when an auction being echoed.
    #[ink(event)]
    pub struct AuctionEchoed {
        auction: Auction,
    }

    /// Storage of the contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Mother {
        auction: Auction,
        balances: Mapping<Address, Balance>,
        log: StorageVec<String>,
    }

    impl Mother {
        #[ink(constructor)]
        pub fn new(auction: Auction) -> Self {
            Self {
                balances: Default::default(),
                log: Default::default(),
                auction,
            }
        }

        #[ink(constructor)]
        pub fn new_default() -> Self {
            Default::default()
        }

        /// Demonstrates the ability to fail a constructor safely.
        #[ink(constructor)]
        pub fn failed_new(fail: bool) -> Result<Self, Failure> {
            if fail {
                Err(Failure::Revert("Reverting instantiation".to_string()))
            } else {
                Ok(Default::default())
            }
        }

        /// Takes an auction data struct as input and returns it back.
        #[ink(message)]
        pub fn echo_auction(&mut self, auction: Auction) -> Auction {
            self.env().emit_event(AuctionEchoed {
                auction: auction.clone(),
            });
            auction
        }

        /// Fails contract execution in the required way.
        #[ink(message)]
        pub fn revert_or_trap(&mut self, fail: Option<Failure>) -> Result<(), Failure> {
            match fail {
                Some(Failure::Revert(_)) => {
                    Err(Failure::Revert("Reverting on user demand!".to_string()))
                }
                Some(Failure::Panic) => {
                    panic!("Trapping on user demand!")
                }
                None => Ok(()),
            }
        }

        /// Mutates the input string to return "Hello, { name }"
        #[ink(message)]
        pub fn mut_hello_world(&self, mut message: String) -> String {
            message = format!("Hello, {message}");
            message
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn echo_auction_works() {
            let auction = Auction::default();
            let mut contract = Mother::new_default();
            assert_eq!(contract.echo_auction(auction.clone()), auction);
        }

        #[ink::test]
        fn revert_works() {
            let mut contract = Mother::default();
            assert_eq!(
                contract.revert_or_trap(Some(Failure::Revert(
                    "Testing reverting on demand!".to_string()
                ))),
                Err(Failure::Revert("Reverting on user demand!".to_string()))
            );
            contract
                .revert_or_trap(None)
                .expect("Contract unexpected failure!");
        }

        #[ink::test]
        fn constructor_works_or_fails() {
            let contract = Mother::failed_new(true);
            assert!(contract.is_err());
            assert_eq!(
                contract.err(),
                Some(Failure::Revert("Reverting instantiation".to_string()))
            );

            let contract = Mother::failed_new(false);
            assert!(contract.is_ok());
        }

        #[ink::test]
        #[should_panic]
        fn trap_works() {
            let mut contract = Mother::default();
            let _ = contract.revert_or_trap(Some(Failure::Panic));
        }

        #[ink::test]
        fn mut_works() {
            let contract = Mother::default();
            let res = contract.mut_hello_world("Alice".to_string());
            assert_eq!("Hello, Alice", res)
        }
    }
}
