#![cfg_attr(not(feature = "std"), no_std)]

mod tests;

#[ink::contract]
mod dao {
    use ink::env::Result as EnvResult;
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        CallFlags, DefaultEnvironment,
    };
    use ink::storage::Mapping;
    use ink::MessageResult;

    type Result<T> = core::result::Result<T, Error>;
    type ProposalId = u64;
    type Votes = u128;

    // For testing purposes I made it minutes.
    const DAYS: u64 = 60 * 1_000;

    // A proposal that can be made with `fn propose`.
    #[derive(scale::Decode, scale::Encode, Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
    pub struct Proposal {
        pub to: AccountId,
        pub amount: Balance,
        pub start: Timestamp,
        pub end: Timestamp,
        pub executed: bool,
    }

    // The amount of votes on a given `Proposal`.
    #[derive(scale::Decode, scale::Encode, Default, Debug, PartialEq, Eq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
    pub struct ProposalVotes {
        pub total_yes: Votes,
        pub total_no: Votes,
    }

    // Type of a vote.
    #[derive(scale::Decode, scale::Encode, Clone, Copy)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
    pub enum VoteType {
        Yes,
        No,
    }

    // Contract storage.
    #[ink(storage)]
    pub struct Dao {
        // Store the proposal to proposal id.
        pub proposals: Mapping<ProposalId, Proposal>,
        // Store the votes (total yes and no) to the proposal id.
        pub proposal_votes: Mapping<ProposalId, ProposalVotes>,
        // Store a voter's account id, when it voted, to the proposal id.
        pub votes: Mapping<(ProposalId, AccountId), ()>,
        // Store the number of voters to a proposal id.
        pub total_voters: Mapping<ProposalId, u8>,
        // The proposal id for the next proposal.
        pub next_proposal_id: ProposalId,
        // The token accounts need to vote.
        pub governance_token: AccountId,
        // The minimul number of voters required for a proposal to be executed.
        pub quorum: u8,
    }

    // #[ink(event)]
    // pub struct DaoCreated {
    //     #[ink(topic)]
    //     pub governance_token: AccountId,
    //     pub quorum: u8,
    // }

    // #[ink(event)]
    // pub struct ProposalCreated {
    //     #[ink(topic)]
    //     pub proposal_id: ProposalId,
    //     #[ink(topic)]
    //     pub to: AccountId,
    //     pub amount: Balance,
    //     pub duration: u64,
    // }

    // #[ink(event)]
    // pub struct Vote {
    //     #[ink(topic)]
    //     pub proposal_id: ProposalId,
    //     #[ink(topic)]
    //     pub who: AccountId,
    //     pub vote_type: VoteType,
    //     pub vote_amount: Votes,
    // }

    // #[ink(event)]
    // pub struct ProposalExecuted {
    //     #[ink(topic)]
    //     pub proposal_id: ProposalId,
    //     #[ink(topic)]
    //     to: AccountId,
    //     amount: Balance,
    //     total_yes: Votes,
    //     total_no: Votes,
    //     total_voters: u8,
    // }

    // #[ink(event)]
    // pub struct Fund {
    //     #[ink(topic)]
    //     from: AccountId,
    //     amount: Balance,
    // }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        // The proposed amount must be higher than 0.
        InvalidProposalAmount,
        // The duration of the proposal is too short.
        InvalidProposalDuration,
        // Proposal ID does not exist.
        ProposalNotFound,
        // Proposal has been executed.
        ProposalExecuted,
        // Proposal has expired.
        ProposalExpired,
        // Voter has already voted on this proposal.
        AlreadyVoted,
        // Overflow.
        ArithmeticOverflow,
        // No tokens to vote.
        InsufficientBalance,
        // Proposal is still active.
        ProposalStillActive,
        // Less people than required has voted on the proposal which makes it invalid.
        QuorumNotMet,
        // Proposal has not been accepted by the majority of the token holders.
        ProposalRefused,
        // Insufficient treasury balance for proposal execution.
        InsufficientTreasuryBalance,
        // Transfer for the execution of the transfer failed.
        TransferFailed,
    }

    impl Dao {
        // Instantiate a new DAO.
        #[ink(constructor, payable)]
        pub fn new(governance_token: AccountId, quorum: u8) -> Self {
            // Self::env().emit_event(DaoCreated {
            //     governance_token,
            //     quorum,
            // });
            Dao {
                proposals: Mapping::default(),
                proposal_votes: Mapping::default(),
                votes: Mapping::default(),
                total_voters: Mapping::default(),
                next_proposal_id: 0,
                governance_token,
                quorum,
            }
        }

        // Propose a new proposal.
        #[ink(message)]
        pub fn propose(
            &mut self,
            to: AccountId,
            amount: Balance,
            duration: u64,
        ) -> Result<()> {
            if amount == 0 || amount > self.env().balance() {
                return Err(Error::InvalidProposalAmount);
            }
            if duration == 0 {
                return Err(Error::InvalidProposalDuration);
            }
            let proposal_id = self.create_proposal_id()?;
            self.next_proposal_id += 1;

            // Create `Proposal`
            let now = self.env().block_timestamp();
            self.proposals.insert(
                proposal_id,
                &Proposal {
                    to,
                    amount,
                    start: now,
                    end: now + (duration * DAYS),
                    executed: false,
                },
            );
            self.proposal_votes.insert(
                proposal_id,
                &ProposalVotes {
                    total_yes: 0,
                    total_no: 0,
                },
            );
            self.total_voters.insert(proposal_id, &0);
            // Self::env().emit_event(ProposalCreated {
            //     proposal_id,
            //     to,
            //     amount,
            //     duration,
            // });
            Ok(())
        }

        #[inline]
        fn create_proposal_id(&mut self) -> Result<u64> {
            self.next_proposal_id
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)
        }

        // Vote on a proposal.
        #[ink(message)]
        pub fn vote(
            &mut self,
            proposal_id: ProposalId,
            vote_type: VoteType,
        ) -> Result<()> {
            let proposal = match self.proposals.get(proposal_id) {
                Some(proposal) => proposal,
                _ => return Err(Error::ProposalNotFound),
            };
            self.proposal_executed(&proposal)?;
            if self.proposal_expired(&proposal) {
                return Err(Error::ProposalExpired);
            }
            let caller = self.env().caller();
            self.has_voted(proposal_id, caller)?;

            let voting_power = self.get_voting_power(caller)?;
            self.add_votes(voting_power, proposal_id, vote_type)?;
            self.votes.insert((&proposal_id, &caller), &());
            // Self::env().emit_event(Vote {
            //     proposal_id,
            //     who: caller,
            //     vote_type,
            //     vote_amount,
            // });
            Ok(())
        }

        #[inline]
        fn get_voting_power(&self, caller: AccountId) -> Result<Votes> {
            let balance = self.balance_of(caller);
            if balance == 0 {
                return Err(Error::InsufficientBalance);
            }
            Ok(balance)
        }

        #[inline]
        fn add_votes(
            &mut self,
            vote_amount: Balance,
            proposal_id: ProposalId,
            vote_type: VoteType,
        ) -> Result<()> {
            let proposal_votes = self
                .proposal_votes
                .get(proposal_id)
                .unwrap_or_else(|| panic!("Developer is a dickhead"));
            let proposal_votes = match vote_type {
                VoteType::Yes => ProposalVotes {
                    total_yes: proposal_votes
                        .total_yes
                        .checked_add(vote_amount)
                        .ok_or(Error::ArithmeticOverflow)?,
                    total_no: proposal_votes.total_no,
                },
                VoteType::No => ProposalVotes {
                    total_yes: proposal_votes.total_yes,
                    total_no: proposal_votes
                        .total_no
                        .checked_add(vote_amount)
                        .ok_or(Error::ArithmeticOverflow)?,
                },
            };
            self.proposal_votes.insert(proposal_id, &proposal_votes);
            let total_voters = self.total_voters.get(proposal_id).unwrap_or_default();
            self.total_voters.insert(proposal_id, &(total_voters + 1));
            Ok(())
        }

        #[inline]
        fn balance_of(&self, caller: AccountId) -> Balance {
            let result = build_call::<DefaultEnvironment>()
                .call(self.governance_token)
                .gas_limit(0)
                .transferred_value(0)
                .call_flags(CallFlags::default())
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!(
                        "balance_of"
                    )))
                    .push_arg(caller),
                )
                .returns::<Balance>()
                .try_invoke();
            match result {
                EnvResult::Ok(MessageResult::Ok(result)) => result,
                _ => unimplemented!(),
            }
        }

        #[inline]
        fn proposal_executed(&self, proposal: &Proposal) -> Result<()> {
            if proposal.executed {
                return Err(Error::ProposalExecuted);
            }
            Ok(())
        }

        #[inline]
        fn proposal_expired(&self, proposal: &Proposal) -> bool {
            if self.env().block_timestamp() >= proposal.end {
                return true;
            }
            false
        }

        #[inline]
        fn has_voted(&self, proposal_id: ProposalId, voter: AccountId) -> Result<()> {
            if self.votes.get((proposal_id, voter)).is_some() {
                return Err(Error::AlreadyVoted);
            }
            Ok(())
        }

        // Execute a proposal.
        #[ink(message)]
        pub fn execute(&mut self, proposal_id: ProposalId) -> Result<()> {
            let mut proposal = match self.proposals.get(proposal_id) {
                Some(proposal) => proposal,
                _ => return Err(Error::ProposalNotFound),
            };
            self.proposal_executed(&proposal)?;
            if !self.proposal_expired(&proposal) {
                return Err(Error::ProposalStillActive);
            }
            let _total_voters = self.quorum_met(proposal_id)?;
            let proposal_votes = match self.proposal_votes.get(proposal_id) {
                Some(proposal_votes) => proposal_votes,
                _ => panic!("Developer is a dickhead"),
            };
            self.proposal_pass(proposal_votes)?;
            let treasury = self.env().balance();
            if proposal.amount > treasury {
                return Err(Error::InsufficientTreasuryBalance);
            }
            self.transfer_proposal_amount(proposal)?;
            proposal.executed = true;
            self.proposals.insert(proposal_id, &proposal);
            // Self::env().emit_event(ProposalExecuted {
            //     proposal_id,
            //     to: proposal.to,
            //     amount: proposal.amount,
            //     total_yes: proposal_votes.total_yes,
            //     total_no: proposal_votes.total_no,
            //     total_voters,
            // });
            Ok(())
        }

        #[inline]
        fn quorum_met(&self, proposal_id: ProposalId) -> Result<u8> {
            let total_voters = self.total_voters.get(proposal_id).unwrap_or_default();
            if total_voters < self.quorum {
                return Err(Error::QuorumNotMet);
            }
            Ok(total_voters)
        }

        #[inline]
        fn proposal_pass(&self, proposal_votes: ProposalVotes) -> Result<()> {
            if proposal_votes.total_yes <= proposal_votes.total_no {
                return Err(Error::ProposalRefused);
            }
            Ok(())
        }

        #[inline]
        fn transfer_proposal_amount(&self, proposal: Proposal) -> Result<()> {
            if self.env().transfer(proposal.to, proposal.amount).is_err() {
                return Err(Error::TransferFailed);
            }
            Ok(())
        }

        #[ink(message, payable)]
        pub fn fund(&self) {
            let _from = self.env().caller();
            let _amount = self.env().transferred_value();
            // Self::env().emit_event(Fund{
            //     from,
            //     amount,
            // });
        }

        #[ink(message)]
        pub fn get_treasury_amount(&self) -> Balance {
            self.env().balance()
        }

        // Get the information regarding a proposal.
        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: ProposalId) -> Result<Proposal> {
            self.proposals
                .get(proposal_id)
                .ok_or(Error::ProposalNotFound)
        }

        // Get the total votes regarding a proposal.
        #[ink(message)]
        pub fn get_votes(&self, proposal_id: ProposalId) -> Result<ProposalVotes> {
            self.proposal_votes
                .get(proposal_id)
                .ok_or(Error::ProposalNotFound)
        }

        // Get the number of voters that voted on a proposal
        #[ink(message)]
        pub fn get_voters(&self, proposal_id: ProposalId) -> Result<u8> {
            self.total_voters
                .get(proposal_id)
                .ok_or(Error::ProposalNotFound)
        }

        // Get the amount of time left to vote on a proposal.
        #[ink(message)]
        pub fn get_proposal_end(&self, proposal_id: ProposalId) -> Result<Timestamp> {
            let proposal = self
                .proposals
                .get(proposal_id)
                .ok_or(Error::ProposalNotFound)?;
            Ok(proposal.end)
        }

        // #[inline]
        // fn total_supply(&self) -> Balance {
        //     build_call::<DefaultEnvironment>()
        //         .call(self.governance_token)
        //         .gas_limit(0)
        //         .transferred_value(0)
        //         .call_flags(CallFlags::default())
        //         .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
        //             "total_supply"
        //         ))))
        //         .returns::<Balance>()
        //         .try_invoke()
        //         .unwrap_or_else(|env_err| {
        //             panic!("cross-contract call to erc20 failed due to {:?}", env_err)
        //         })
        //         .unwrap_or_else(|lang_err| {
        //             panic!("cross-contract call to erc20 failed due to {:?}", lang_err)
        //         })
        // }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use erc20::Erc20Ref;
        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn gets_work(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate dao contract
            let governance_token: AccountId = [0x08; 32].into();
            let quorum = 10;
            let dao_constructor = DaoRef::new(governance_token, quorum);
            let dao_id = client
                .instantiate("dao", &ink_e2e::ferdie(), dao_constructor, 100, None)
                .await
                .expect("dao contract instantiation failed")
                .account_id;
            // Get treasury balance
            let get_treasury_amount = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_treasury_amount());
            let get_treasury_amount_result = client
                .call_dry_run(&ink_e2e::alice(), &get_treasury_amount, 0, None)
                .await;
            assert_eq!(get_treasury_amount_result.return_value(), 100);
            // Test get_* without a proposal:
            //
            // Get end of proposal
            let get_end = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_proposal_end(1));
            let get_end_result = client
                .call_dry_run(&ink_e2e::alice(), &get_end, 0, None)
                .await;
            assert_eq!(get_end_result.return_value(), Err(Error::ProposalNotFound));
            // Get proposal
            let get_proposal = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_proposal(1));
            let get_proposal_result = client
                .call_dry_run(&ink_e2e::alice(), &get_proposal, 0, None)
                .await;
            assert_eq!(
                get_proposal_result.return_value(),
                Err(Error::ProposalNotFound)
            );
            // Get total votes on proposal
            let get_votes = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_votes(1));
            let get_votes_result = client
                .call_dry_run(&ink_e2e::alice(), &get_votes, 0, None)
                .await;
            assert_eq!(
                get_votes_result.return_value(),
                Err(Error::ProposalNotFound)
            );
            // Test with a proposal:
            //
            // Propose a proposal
            let ferdie_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie);
            let propose_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.propose(ferdie_account.clone(), 10, 10));
            let _propose_result = client
                .call(&ink_e2e::alice(), propose_message, 0, None)
                .await
                .expect("Proposal failed");
            // Get end of proposal
            let get_end = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_proposal_end(1));
            let get_end_result = client
                .call_dry_run(&ink_e2e::alice(), &get_end, 0, None)
                .await;
            assert!(get_end_result.exec_result.result.is_ok());
            // Hacky way due to no timestamp() in e2e_tests yet
            let start = get_end_result
                .return_value()
                .unwrap_or_else(|_| panic!("shouldn't panic"))
                - (10 * DAYS);
            // Get proposal
            let get_proposal = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_proposal(1));
            let get_proposal_result = client
                .call_dry_run(&ink_e2e::alice(), &get_proposal, 0, None)
                .await;
            assert_eq!(
                get_proposal_result.return_value(),
                Ok(Proposal {
                    to: ferdie_account,
                    amount: 10,
                    start,
                    end: start + (10 * DAYS),
                    executed: false,
                })
            );
            // Get total voters on proposal
            let get_voters = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_voters(1));
            let get_voters_result = client
                .call_dry_run(&ink_e2e::alice(), &get_voters, 0, None)
                .await;
            assert_eq!(get_voters_result.return_value(), Ok(0));

            // Get total votes on proposal
            let get_votes = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_votes(1));
            let get_votes_result = client
                .call_dry_run(&ink_e2e::alice(), &get_votes, 0, None)
                .await;
            assert_eq!(
                get_votes_result.return_value(),
                Ok(ProposalVotes {
                    total_yes: 0,
                    total_no: 0,
                })
            );

            // Get total voters on proposal
            let get_voters = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_voters(1));
            let get_voters_result = client
                .call_dry_run(&ink_e2e::alice(), &get_voters, 0, None)
                .await;
            assert_eq!(get_voters_result.return_value(), Ok(0));
            Ok(())
        }

        #[ink_e2e::test]
        async fn incorrect_proposals(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate dao contract
            let governance_token: AccountId = [0x08; 32].into();
            let quorum = 10;
            let dao_constructor = DaoRef::new(governance_token, quorum);
            let dao_id = client
                .instantiate("dao", &ink_e2e::ferdie(), dao_constructor, 100, None)
                .await
                .expect("dao contract instantiation failed")
                .account_id;
            // Get treasury balance
            let get_treasury_amount = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_treasury_amount());
            let get_treasury_amount_result = client
                .call_dry_run(&ink_e2e::alice(), &get_treasury_amount, 0, None)
                .await;
            assert_eq!(get_treasury_amount_result.return_value(), 100);
            // Propose an incorrect proposals
            //
            // Invalid proposal amount (Error::InvalidProposalAmount)
            let ferdie_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie);
            let propose_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.propose(ferdie_account.clone(), 0, 10));
            let propose_result = client
                .call(&ink_e2e::alice(), propose_message, 0, None)
                .await;
            assert!(propose_result.is_err());
            // Invalid proposal duration (Error::InvalidProposalDuration)
            let propose_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.propose(ferdie_account.clone(), 10, 0));
            let propose_result = client
                .call(&ink_e2e::alice(), propose_message, 0, None)
                .await;
            assert!(propose_result.is_err());

            // Get checks
            let get_end = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_proposal_end(1));
            let get_end_result = client
                .call_dry_run(&ink_e2e::alice(), &get_end, 0, None)
                .await;
            assert_eq!(get_end_result.return_value(), Err(Error::ProposalNotFound));
            let get_proposal = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_proposal(1));
            let get_proposal_result = client
                .call_dry_run(&ink_e2e::alice(), &get_proposal, 0, None)
                .await;
            assert_eq!(
                get_proposal_result.return_value(),
                Err(Error::ProposalNotFound)
            );
            let get_votes = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_votes(1));
            let get_votes_result = client
                .call_dry_run(&ink_e2e::alice(), &get_votes, 0, None)
                .await;
            assert_eq!(
                get_votes_result.return_value(),
                Err(Error::ProposalNotFound)
            );

            // Get total voters on proposal
            let get_voters = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_voters(1));
            let get_voters_result = client
                .call_dry_run(&ink_e2e::alice(), &get_voters, 0, None)
                .await;
            assert_eq!(
                get_voters_result.return_value(),
                Err(Error::ProposalNotFound)
            );
            Ok(())
        }

        #[ink_e2e::test]
        async fn instantiate_erc20(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate erc20 contract
            let total_supply = 1_000;
            let erc20_constructor = Erc20Ref::new(total_supply);
            let erc20_acc_id = client
                .instantiate("erc20", &ink_e2e::alice(), erc20_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Transfer tokens to Bob
            let bob_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Bob);
            let transfer_to_bob = 200u128;
            let transfer = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.transfer(bob_account.clone(), transfer_to_bob));
            let _transfer_res = client
                .call(&ink_e2e::alice(), transfer, 0, None)
                .await
                .expect("transfer failed");

            // Get balance of Bob
            let balance_of = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.balance_of(bob_account));
            let balance_of_res = client
                .call_dry_run(&ink_e2e::alice(), &balance_of, 0, None)
                .await;

            // Get total supply
            let total_supply_msg = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.total_supply());
            let total_supply_res = client
                .call_dry_run(&ink_e2e::bob(), &total_supply_msg, 0, None)
                .await;

            assert_eq!(
                total_supply,
                total_supply_res.return_value(),
                "total_supply"
            );
            assert_eq!(transfer_to_bob, balance_of_res.return_value(), "balance_of");
            Ok(())
        }

        #[ink_e2e::test]
        async fn correct_voting(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate erc20 contract
            let total_supply = 1_000;
            let erc20_constructor = Erc20Ref::new(total_supply);
            let erc20_acc_id = client
                .instantiate("erc20", &ink_e2e::alice(), erc20_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
            // Instantiate dao contract
            let quorum = 10;
            let dao_constructor = DaoRef::new(erc20_acc_id, quorum);
            let dao_id = client
                .instantiate("dao", &ink_e2e::ferdie(), dao_constructor, 100, None)
                .await
                .expect("dao contract instantiation failed")
                .account_id;
            // Transfer tokens to Bob
            let bob_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Bob);
            let transfer_to_bob = 200u128;
            let transfer = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.transfer(bob_account.clone(), transfer_to_bob));
            let _transfer_res = client
                .call(&ink_e2e::alice(), transfer, 0, None)
                .await
                .expect("transfer failed");

            // Get balance of Bob
            let balance_of = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.balance_of(bob_account));
            let balance_of_res = client
                .call_dry_run(&ink_e2e::alice(), &balance_of, 0, None)
                .await;
            assert_eq!(transfer_to_bob, balance_of_res.return_value());

            // Transfer tokens to Ferdie
            let ferdie_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie);
            let transfer_to_ferdie = 100u128;
            let transfer = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.transfer(ferdie_account.clone(), transfer_to_ferdie));
            let _transfer_res = client
                .call(&ink_e2e::alice(), transfer, 0, None)
                .await
                .expect("transfer failed");

            // Get balance of Ferdie
            let balance_of = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.balance_of(ferdie_account));
            let balance_of_res = client
                .call_dry_run(&ink_e2e::alice(), &balance_of, 0, None)
                .await;
            assert_eq!(transfer_to_ferdie, balance_of_res.return_value());

            // Propose a proposal
            let ferdie_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie);
            let propose_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.propose(ferdie_account.clone(), 10, 10));
            let propose_result = client
                .call(&ink_e2e::alice(), propose_message, 0, None)
                .await;
            assert!(propose_result.is_ok());

            // // Alice votes `yes`
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(1, VoteType::Yes));
            let _vote_result = client
                .call(&ink_e2e::alice(), vote_message, 0, None)
                .await
                .expect("vote failed");
            // Ferdie votes `yes`
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(1, VoteType::Yes));
            let _vote_result = client
                .call(&ink_e2e::ferdie(), vote_message, 0, None)
                .await
                .expect("vote failed");
            // Bob votes `no`
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(1, VoteType::No));
            let _vote_result = client
                .call(&ink_e2e::bob(), vote_message, 0, None)
                .await
                .expect("vote failed");

            // Get total votes on proposal
            let get_votes = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_votes(1));
            let get_votes_result = client
                .call_dry_run(&ink_e2e::alice(), &get_votes, 0, None)
                .await;
            assert_eq!(
                get_votes_result.return_value(),
                Ok(ProposalVotes {
                    total_yes: 800,
                    total_no: 200,
                })
            );

            // Get total voters on proposal
            let get_voters = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_voters(1));
            let get_voters_result = client
                .call_dry_run(&ink_e2e::alice(), &get_voters, 0, None)
                .await;
            assert_eq!(get_voters_result.return_value(), Ok(3));
            Ok(())
        }

        #[ink_e2e::test]
        async fn incorrect_voting(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate erc20 contract
            let total_supply = 1_000;
            let erc20_constructor = Erc20Ref::new(total_supply);
            let erc20_acc_id = client
                .instantiate("erc20", &ink_e2e::alice(), erc20_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
            // Instantiate dao contract
            let quorum = 10;
            let dao_constructor = DaoRef::new(erc20_acc_id, quorum);
            let dao_id = client
                .instantiate("dao", &ink_e2e::ferdie(), dao_constructor, 100, None)
                .await
                .expect("dao contract instantiation failed")
                .account_id;
            // Transfer tokens to Bob
            let bob_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Bob);
            let transfer_to_bob = 200u128;
            let transfer = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.transfer(bob_account.clone(), transfer_to_bob));
            let _transfer_res = client
                .call(&ink_e2e::alice(), transfer, 0, None)
                .await
                .expect("transfer failed");

            // Get balance of Bob
            let balance_of = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.balance_of(bob_account));
            let balance_of_res = client
                .call_dry_run(&ink_e2e::alice(), &balance_of, 0, None)
                .await;
            assert_eq!(transfer_to_bob, balance_of_res.return_value());

            // Propose a proposal
            let ferdie_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie);
            let propose_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.propose(ferdie_account.clone(), 10, 10));
            let propose_result = client
                .call(&ink_e2e::alice(), propose_message, 0, None)
                .await;
            assert!(propose_result.is_ok());

            // Get balance of Ferdie (= 0)
            let balance_of = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.balance_of(ferdie_account));
            let balance_of_res = client
                .call_dry_run(&ink_e2e::alice(), &balance_of, 0, None)
                .await;
            assert_eq!(balance_of_res.return_value(), 0);

            // Ferdie votes `no` without balance (Error::InsufficientBalance)
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(1, VoteType::No));
            let vote_result =
                client.call(&ink_e2e::ferdie(), vote_message, 0, None).await;
            assert!(vote_result.is_err());

            // Alice votes `yes`
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(1, VoteType::Yes));
            let _vote_result = client
                .call(&ink_e2e::alice(), vote_message, 0, None)
                .await
                .expect("vote failed");

            // Alice votes `yes` again (Error::AlreadyVoted)
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(1, VoteType::Yes));
            let vote_result = client.call(&ink_e2e::alice(), vote_message, 0, None).await;
            assert!(vote_result.is_err());

            // Alice votes `no` again (Error::AlreadyVoted)
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(1, VoteType::No));
            let vote_result = client.call(&ink_e2e::alice(), vote_message, 0, None).await;
            assert!(vote_result.is_err());

            // Bob votes `no` on non existing proposal (Error::ProposalNotFound)
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(2, VoteType::No));
            let vote_result = client.call(&ink_e2e::bob(), vote_message, 0, None).await;
            assert!(vote_result.is_err());

            // Get total votes on proposal
            let get_votes = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_votes(1));
            let get_votes_result = client
                .call_dry_run(&ink_e2e::alice(), &get_votes, 0, None)
                .await;
            assert_eq!(
                get_votes_result.return_value(),
                Ok(ProposalVotes {
                    total_yes: 800,
                    total_no: 0,
                })
            );

            // Get total voters on proposal
            let get_voters = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_voters(1));
            let get_voters_result = client
                .call_dry_run(&ink_e2e::alice(), &get_voters, 0, None)
                .await;
            assert_eq!(get_voters_result.return_value(), Ok(1));
            Ok(())
        }

        #[ink_e2e::test]
        async fn incorrect_proposal_execution(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // Instantiate erc20 contract
            let total_supply = 1_000;
            let erc20_constructor = Erc20Ref::new(total_supply);
            let erc20_acc_id = client
                .instantiate("erc20", &ink_e2e::alice(), erc20_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
            // Instantiate dao contract
            let quorum = 10;
            let dao_constructor = DaoRef::new(erc20_acc_id, quorum);
            let dao_id = client
                .instantiate("dao", &ink_e2e::ferdie(), dao_constructor, 100, None)
                .await
                .expect("dao contract instantiation failed")
                .account_id;
            // Transfer tokens to Bob
            let bob_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Bob);
            let transfer_to_bob = 200u128;
            let transfer = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.transfer(bob_account.clone(), transfer_to_bob));
            let _transfer_res = client
                .call(&ink_e2e::alice(), transfer, 0, None)
                .await
                .expect("transfer failed");

            // Get balance of Bob
            let balance_of = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.balance_of(bob_account));
            let balance_of_res = client
                .call_dry_run(&ink_e2e::alice(), &balance_of, 0, None)
                .await;
            assert_eq!(transfer_to_bob, balance_of_res.return_value());

            // Transfer tokens to Ferdie
            let ferdie_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie);
            let transfer_to_ferdie = 100u128;
            let transfer = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.transfer(ferdie_account.clone(), transfer_to_ferdie));
            let _transfer_res = client
                .call(&ink_e2e::alice(), transfer, 0, None)
                .await
                .expect("transfer failed");

            // Get balance of Ferdie
            let balance_of = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|erc20| erc20.balance_of(ferdie_account));
            let balance_of_res = client
                .call_dry_run(&ink_e2e::alice(), &balance_of, 0, None)
                .await;
            assert_eq!(transfer_to_ferdie, balance_of_res.return_value());

            // Propose a proposal
            let ferdie_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie);
            let propose_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.propose(ferdie_account.clone(), 10, 10));
            let propose_result = client
                .call(&ink_e2e::alice(), propose_message, 0, None)
                .await;
            assert!(propose_result.is_ok());

            // // Alice votes `yes`
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(1, VoteType::Yes));
            let _vote_result = client
                .call(&ink_e2e::alice(), vote_message, 0, None)
                .await
                .expect("vote failed");
            // Ferdie votes `no`
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(1, VoteType::No));
            let _vote_result = client
                .call(&ink_e2e::ferdie(), vote_message, 0, None)
                .await
                .expect("vote failed");
            // Bob votes `no`
            let vote_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.vote(1, VoteType::No));
            let _vote_result = client
                .call(&ink_e2e::bob(), vote_message, 0, None)
                .await
                .expect("vote failed");

            // Get total votes on proposal
            let get_votes = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_votes(1));
            let get_votes_result = client
                .call_dry_run(&ink_e2e::alice(), &get_votes, 0, None)
                .await;
            assert_eq!(
                get_votes_result.return_value(),
                Ok(ProposalVotes {
                    total_yes: 700,
                    total_no: 300,
                })
            );

            // Get total voters on proposal
            let get_voters = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_voters(1));
            let get_voters_result = client
                .call_dry_run(&ink_e2e::alice(), &get_voters, 0, None)
                .await;
            assert_eq!(get_voters_result.return_value(), Ok(3));

            // Execute proposal (Error::ProposalStillActive)
            let execute_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.execute(1));
            let execute_result =
                client.call(&ink_e2e::bob(), execute_message, 0, None).await;
            assert!(execute_result.is_err());
            Ok(())
        }

        #[ink_e2e::test]
        async fn fund(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate erc20 contract
            let total_supply = 1_000;
            let erc20_constructor = Erc20Ref::new(total_supply);
            let erc20_acc_id = client
                .instantiate("erc20", &ink_e2e::alice(), erc20_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
            // Instantiate dao contract
            let quorum = 10;
            let dao_constructor = DaoRef::new(erc20_acc_id, quorum);
            let dao_id = client
                .instantiate("dao", &ink_e2e::ferdie(), dao_constructor, 100, None)
                .await
                .expect("dao contract instantiation failed")
                .account_id;
            // Get treasury balance (100)
            let get_treasury_amount = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_treasury_amount());
            let get_treasury_amount_result = client
                .call_dry_run(&ink_e2e::alice(), &get_treasury_amount, 0, None)
                .await;
            assert_eq!(get_treasury_amount_result.return_value(), 100);
            // Fund the dao contract (50)
            let fund_message =
                ink_e2e::build_message::<DaoRef>(dao_id.clone()).call(|dao| dao.fund());
            let _fund_result = client
                .call(&ink_e2e::bob(), fund_message, 50, None)
                .await
                .expect("fund failed");
            // Get treasury balance (150)
            let get_treasury_amount = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_treasury_amount());
            let get_treasury_amount_result = client
                .call_dry_run(&ink_e2e::alice(), &get_treasury_amount, 0, None)
                .await;
            assert_eq!(get_treasury_amount_result.return_value(), 150);
            Ok(())
        }

        // Things that is not possible to test;
        // - Vote on a proposal which has expired (Error::ProposalExpired).
        // - Vote on a proposal which already has been executed (Error::ProposalExecuted).
        // - Execute a proposal successfully.
        // - Execute a proposal while the quorum is not met (Error::QuorumNotMet).
        // - Execute a proposal which hasn't been accepted by the majority of the voters (Error::ProposalRefused).
        // - Execute a proposal with insufficient treasury balance (Error::InsufficientTreasuryBalance).
    }
}
