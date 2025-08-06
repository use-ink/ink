//! A smart contract which demonstrates functionality of `lazyvec` functions.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod lazyvec {
    use ink::{
        prelude::vec::Vec,
        storage::StorageVec,
    };

    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Proposal {
        data: Vec<u8>,
        until: BlockNumber,
        approvals: u32,
        min_approvals: u32,
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

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn create_and_vote<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = LazyVectorRef::default();
            let contract = client
                .instantiate("lazyvec", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<LazyVector>();

            // when
            let create = call_builder.create_proposal(vec![0x41], 15, 1);
            let _ = client
                .call(&ink_e2e::alice(), &create)
                .submit()
                .await
                .expect("Calling `create_proposal` failed");

            let approve = call_builder.approve();
            let _ = client
                .call(&ink_e2e::alice(), &approve)
                .submit()
                .await
                .expect("Voting failed");
            let _ = client
                .call(&ink_e2e::bob(), &approve)
                .submit()
                .await
                .expect("Voting failed");

            // then
            let value = client
                .call(&ink_e2e::alice(), &create)
                .dry_run()
                .await
                .expect("create trapped when it shouldn't")
                .return_value();
            assert_eq!(value, None);

            let value = client
                .call(&ink_e2e::alice(), &call_builder.get(0))
                .dry_run()
                .await
                .expect("get trapped when it shouldn't")
                .return_value();
            assert_eq!(value.unwrap().approvals, 2);

            Ok(())
        }
    }
}
