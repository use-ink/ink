//! # The Mother of All Contracts
//!
//! This contract is intended to make use of all features that are observable
//! by off-chain tooling (for example user interfaces).
//! It doesn't do anything useful beyond serving off-chain tooling developers
//! with a contract to test their software against.
//!
//! Currently, this includes the following:
//!
//!   1. Use complex nested input and output types.
//!      This is done with the use case of a data structure
//!      needed to store data of a candle auction.
//!   2. Make contract fail with `ContractTrapped`.
//!   3. Make contract fail with returning an `Error`.
//!   4. Perform debug printing from contract into the node's log.
//!   5. Use complex types in storage.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod mother {
    use ink_prelude::{
        string::{
            String,
            ToString,
        },
        vec::Vec,
    };

    use ink_lang::utils::initialize_contract;
    use ink_storage::{
        traits::{
            PackedLayout,
            SpreadAllocate,
            SpreadLayout,
        },
        Mapping,
    };

    use ink_storage::traits::KeyPtr;
    /// Struct for storing winning bids per bidding sample (a block).
    /// Vector index corresponds to sample number.
    /// Wrapping vector, just added for testing UI components.
    #[derive(
        Default,
        scale::Encode,
        scale::Decode,
        PartialEq,
        Eq,
        Debug,
        Clone,
        SpreadLayout,
        PackedLayout,
        SpreadAllocate,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout,)
    )]
    pub struct Bids(Vec<Vec<Option<(AccountId, Balance)>>>);

    /// Auction outline.
    #[derive(
        scale::Encode,
        scale::Decode,
        Eq,
        PartialEq,
        Debug,
        Clone,
        SpreadLayout,
        PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout,)
    )]
    pub enum Outline {
        NoWinner,
        WinnerDetected,
        PayoutCompleted,
    }

    /// Auction statuses.
    /// Logic inspired by
    /// [Parachain Auction](https://github.com/paritytech/polkadot/blob/master/runtime/common/src/traits.rs#L160)
    #[derive(
        scale::Encode,
        scale::Decode,
        Eq,
        PartialEq,
        Debug,
        Clone,
        SpreadLayout,
        PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout,)
    )]
    pub enum Status {
        /// An auction has not started yet.
        NotStarted,
        /// We are in the starting period of the auction, collecting initial bids.
        OpeningPeriod,
        /// We are in the ending period of the auction, where we are taking snapshots of the winning
        /// bids. Snapshots are taken currently on per-block basis, but this logic could be later evolve
        /// to take snapshots of on arbitrary length (in blocks).
        EndingPeriod(BlockNumber),
        /// Candle was blown.
        Ended(Outline),
        /// We have completed the bidding process and are waiting for the Random Function to return some acceptable
        /// randomness to select the winner. The number represents how many blocks we have been waiting.
        RfDelay(BlockNumber),
    }

    impl SpreadAllocate for Status {
        #[inline]
        fn allocate_spread(ptr: &mut KeyPtr) -> Self {
            ptr.advance_by(<BlockNumber>::FOOTPRINT * 2);
            Self::NotStarted
        }
    }
    /// Struct for storing auction data.
    #[derive(
        Debug,
        PartialEq,
        scale::Encode,
        scale::Decode,
        Clone,
        SpreadLayout,
        PackedLayout,
        SpreadAllocate,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout,)
    )]
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
    #[derive(scale::Encode, scale::Decode, Debug, Eq, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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
    #[derive(Default, SpreadAllocate)]
    pub struct Mother {
        auction: Auction,
        balances: Mapping<AccountId, Balance>,
    }

    impl Mother {
        #[ink(constructor)]
        pub fn new(auction: Auction) -> Self {
            initialize_contract(|c: &mut Self| {
                c.balances = <Mapping<AccountId, Balance>>::default();
                c.auction = auction;
            })
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            initialize_contract(|c: &mut Self| {
                c.balances = <Mapping<AccountId, Balance>>::default();
                c.auction = Auction::default();
            })
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

        /// Prints the specified string into node's debug log.
        #[ink(message)]
        pub fn debug_log(&mut self, _message: String) {
            ink_env::debug_println!("debug_log: {}", _message);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn echo_auction_works() {
            let auction = Auction::default();
            let mut contract = Mother::default();
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
        #[should_panic]
        fn trap_works() {
            let mut contract = Mother::default();
            let _ = contract.revert_or_trap(Some(Failure::Panic));
        }
    }
}
