//! A smart contract which demonstrates functionality of `lazyvec` functions.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod lazyvec {
    use ink::{
        prelude::vec::Vec,
        storage::StorageVec,
    };

    #[ink::storage_item(packed)]
    pub struct Proposal {
        pub data: Vec<u8>,
        pub until: BlockNumber,
        pub approvals: u32,
        pub min_approvals: u32,
    }

    impl Proposal {
        fn is_finished(&self) -> bool {
            self.until < ink::env::block_number::<Environment>()
        }
    }

    #[ink(storage)]
    pub struct LazyVector {
        proposals: StorageVec<Proposal>,
    }

    impl LazyVector {
        #[ink(constructor, payable)]
        pub fn default() -> Self {
            Self {
                proposals: Default::default(),
            }
        }

        /// Checks whether given account is allowed to vote and didn't already
        /// participate.
        fn is_eligible(&self, _voter: Address) -> bool {
            // ToDo: In production, the contract would actually verify eligible voters.
            // For example, a merkle proof could be an efficient way to do this.
            true
        }

        /// Vote to approve the current proposal.
        #[ink(message)]
        pub fn approve(&mut self) {
            assert!(self.is_eligible(self.env().caller()));

            if let Some(mut proposal) = self.proposals.pop() {
                assert!(!proposal.is_finished());

                proposal.approvals = proposal.approvals.saturating_add(1);
                self.proposals.push(&proposal);
            }
        }

        /// Create a new proposal.
        ///
        /// Returns `None` if the current proposal is not yet finished.
        #[ink(message)]
        pub fn create_proposal(
            &mut self,
            data: Vec<u8>,
            duration: BlockNumber,
            min_approvals: u32,
        ) -> Option<u32> {
            let proposal_number = match self.proposals.peek() {
                Some(last) if !last.is_finished() => return None,
                _ => self.proposals.len(),
            };

            self.proposals.push(&Proposal {
                data,
                until: self.env().block_number().saturating_add(duration.min(6000)),
                min_approvals,
                approvals: 0,
            });

            Some(proposal_number)
        }

        #[ink(message)]
        pub fn get(&self, at: u32) -> Option<Proposal> {
            self.proposals.get(at)
        }
    }
}

#[cfg(test)]
mod tests;