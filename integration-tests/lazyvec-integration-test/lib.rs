//! A smart contract which demonstrates functionality of `lazyvec` functions.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod lazyvec_integration_tests {
    use ink::storage::StorageVec;

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
            self.until >= ink::env::block_number::<Environment>()
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

        fn is_eligible(&self, _voter: AccountId) -> bool {
            // todo
            true
        }

        #[ink(message)]
        pub fn approve(&mut self) {
            if !self.is_eligible(self.env().caller()) {
                return;
            }

            let mut proposal = self.proposals.pop().unwrap();

            if proposal.is_finished() {
                return;
            }

            proposal.approvals = proposal.approvals.saturating_add(1);

            self.proposals.push(&proposal);
        }

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
                until: self.env().block_number().saturating_add(duration),
                min_approvals,
                approvals: 0,
            });

            Some(proposal_number)
        }

        #[ink(message)]
        pub fn get(&self, proposal_no: u32) -> Option<Proposal> {
            self.proposals.get(proposal_no)
        }

        #[ink(message)]
        pub fn was_approved(&self, proposal_no: u32) -> Option<bool> {
            self.proposals.get(proposal_no).map(|proposal| {
                proposal.is_finished() && proposal.approvals >= proposal.min_approvals
            })
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn push_and_pop_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = LazyVectorRef::default();
            let contract = client
                .instantiate(
                    "lazyvec-integration-tests",
                    &ink_e2e::alice(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<LazyVector>();

            // when
            let insert = call.push(0);
            let _ = client
                .call(&ink_e2e::alice(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed");

            // then
            let value = client
                .call(&ink_e2e::alice(), &call.pop())
                .dry_run()
                .await
                .return_value();
            assert_eq!(value, Some(0));

            client
                .call(&ink_e2e::alice(), &call.pop())
                .submit()
                .await
                .unwrap();

            let value = client
                .call(&ink_e2e::alice(), &call.pop())
                .dry_run()
                .await
                .return_value();
            assert_eq!(value, None);

            Ok(())
        }
    }
}
