//! # Multisig Wallet
//!
//! This implements a plain multi owner wallet.
//!
//! ## Warning
//!
//! This contract is an *example*. It is neither audited nor endorsed for production use.
//! Do **not** rely on it to keep anything of value secure.
//!
//! ## Overview
//!
//! Each instantiation of this contract has a set of `owners` and a `requirement` of
//! how many of them need to agree on a `Transaction` for it to be able to be executed.
//! Every owner can submit a transaction and when enough of the other owners confirm
//! it will be able to be executed. The following invariant is enforced by the contract:
//!
//! ```ignore
//! 0 < requirement && requirement <= owners && owners <= MAX_OWNERS
//! ```
//!
//! ## Error Handling
//!
//! With the exception of `execute_transaction` no error conditions are signalled
//! through return types. Any error or invariant violation triggers a panic and therefore
//! rolls back the transaction.
//!
//! ## Interface
//!
//! The interface is modelled after the popular Gnosis multisig wallet. However, there
//! are subtle variations from the interface. For example the `confirm_transaction`
//! will never trigger the execution of a `Transaction` even if the threshold is reached.
//! A call of `execute_transaction` is always required. This can be called by anyone.
//!
//! All the messages that are declared as only callable by the wallet must go through
//! the usual submit, confirm, execute cycle as any other transaction that should be
//! called by the wallet. For example, to add an owner you would submit a transaction
//! that calls the wallets own `add_owner` message through `submit_transaction`.
//!
//! ### Owner Management
//!
//! The messages `add_owner`, `remove_owner`, and `replace_owner` can be used to manage
//! the owner set after instantiation.
//!
//! ### Changing the Requirement
//!
//! `change_requirement` can be used to tighten or relax the `requirement` of how many
//! owner signatures are needed to execute a `Transaction`.
//!
//! ### Transaction Management
//!
//! `submit_transaction`, `cancel_transaction`, `confirm_transaction`,
//! `revoke_confirmation` and `execute_transaction` are the bread and butter messages
//! of this contract. Use them to dispatch arbitrary messages to other contracts
//! with the wallet as a sender.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::multisig::{
    ConfirmationStatus,
    Multisig,
    Transaction,
};

#[ink::contract]
mod multisig {
    use ink::{
        U256,
        env::{
            CallFlags,
            call::{
                ExecutionInput,
                build_call,
            },
        },
        prelude::vec::Vec,
        scale::Output,
        storage::Mapping,
    };

    /// Tune this to your liking but be wary that allowing too many owners will not
    /// perform well.
    const MAX_OWNERS: u32 = 50;

    type TransactionId = u32;
    const WRONG_TRANSACTION_ID: &str =
        "The user specified an invalid transaction id. Abort.";

    /// A wrapper that allows us to encode a blob of bytes.
    ///
    /// We use this to pass the set of untyped (bytes) parameters to the `CallBuilder`.
    #[derive(Clone)]
    struct CallInput<'a>(&'a [u8]);

    impl ink::scale::Encode for CallInput<'_> {
        fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
            dest.write(self.0);
        }
    }

    /// Indicates whether a transaction is already confirmed or needs further
    /// confirmations.
    #[derive(Clone, Copy)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum ConfirmationStatus {
        /// The transaction is already confirmed.
        Confirmed,
        /// Indicates how many confirmations are remaining.
        ConfirmationsNeeded(u32),
    }

    /// A Transaction is what every `owner` can submit for confirmation by other owners.
    /// If enough owners agree it will be executed by the contract.
    #[derive(Clone)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
    )]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Transaction {
        /// The address of the contract that is called in this transaction.
        pub callee: Address,
        /// The selector bytes that identifies the function of the callee that should be
        /// called.
        pub selector: [u8; 4],
        /// The SCALE encoded parameters that are passed to the called function.
        pub input: Vec<u8>,
        /// The amount of chain balance that is transferred to the callee.
        pub transferred_value: U256,
        /// Gas limit for the execution of the call.
        pub ref_time_limit: u64,
        /// If set to true the transaction will be allowed to re-enter the multisig
        /// contract. Re-entrancy can lead to vulnerabilities. Use at your own
        /// risk.
        pub allow_reentry: bool,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Returned if the call failed.
        TransactionFailed,
    }

    /// This is a book keeping struct that stores a list of all transaction ids and
    /// also the next id to use. We need it for cleaning up the storage.
    #[derive(Clone, Default)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
    )]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Transactions {
        /// Just store all transaction ids packed.
        transactions: Vec<TransactionId>,
        /// We just increment this whenever a new transaction is created.
        /// We never decrement or defragment. For now, the contract becomes defunct
        /// when the ids are exhausted.
        next_id: TransactionId,
    }

    /// Emitted when an owner confirms a transaction.
    #[ink(event)]
    pub struct Confirmation {
        /// The transaction that was confirmed.
        #[ink(topic)]
        transaction: TransactionId,
        /// The owner that sent the confirmation.
        #[ink(topic)]
        from: Address,
        /// The confirmation status after this confirmation was applied.
        #[ink(topic)]
        status: ConfirmationStatus,
    }

    /// Emitted when an owner revoked a confirmation.
    #[ink(event)]
    pub struct Revocation {
        /// The transaction that was revoked.
        #[ink(topic)]
        transaction: TransactionId,
        /// The owner that sent the revocation.
        #[ink(topic)]
        from: Address,
    }

    /// Emitted when an owner submits a transaction.
    #[ink(event)]
    pub struct Submission {
        /// The transaction that was submitted.
        #[ink(topic)]
        transaction: TransactionId,
    }

    /// Emitted when a transaction was canceled.
    #[ink(event)]
    pub struct Cancellation {
        /// The transaction that was canceled.
        #[ink(topic)]
        transaction: TransactionId,
    }

    /// Emitted when a transaction was executed.
    #[ink(event)]
    pub struct Execution {
        /// The transaction that was executed.
        #[ink(topic)]
        transaction: TransactionId,
        /// Indicates whether the transaction executed successfully. If so the `Ok` value
        /// holds the output in bytes. The Option is `None` when the transaction
        /// was executed through `invoke_transaction` rather than
        /// `evaluate_transaction`.
        #[ink(topic)]
        result: Result<Option<Vec<u8>>, Error>,
    }

    /// Emitted when an owner is added to the wallet.
    #[ink(event)]
    pub struct OwnerAddition {
        /// The owner that was added.
        #[ink(topic)]
        owner: Address,
    }

    /// Emitted when an owner is removed from the wallet.
    #[ink(event)]
    pub struct OwnerRemoval {
        /// The owner that was removed.
        #[ink(topic)]
        owner: Address,
    }

    /// Emitted when the requirement changed.
    #[ink(event)]
    pub struct RequirementChange {
        /// The new requirement value.
        new_requirement: u32,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Multisig {
        /// Every entry in this map represents the confirmation of an owner for a
        /// transaction. This is effectively a set rather than a map.
        confirmations: Mapping<(TransactionId, Address), ()>,
        /// The amount of confirmations for every transaction. This is a redundant
        /// information and is kept in order to prevent iterating through the
        /// confirmation set to check if a transaction is confirmed.
        confirmation_count: Mapping<TransactionId, u32>,
        /// Map the transaction id to its not-executed transaction.
        transactions: Mapping<TransactionId, Transaction>,
        /// We need to hold a list of all transactions so that we can clean up storage
        /// when an owner is removed.
        transaction_list: Transactions,
        /// The list is a vector because iterating over it is necessary when cleaning
        /// up the confirmation set.
        owners: Vec<Address>,
        /// Redundant information to speed up the check whether a caller is an owner.
        is_owner: Mapping<Address, ()>,
        /// Minimum number of owners that have to confirm a transaction to be executed.
        requirement: u32,
    }

    impl Multisig {
        /// The only constructor of the contract.
        ///
        /// A list of owners must be supplied and a number of how many of them must
        /// confirm a transaction. Duplicate owners are silently dropped.
        ///
        /// # Panics
        ///
        /// If `requirement` violates our invariant.
        #[ink(constructor)]
        pub fn new(requirement: u32, mut owners: Vec<Address>) -> Self {
            let mut contract = Multisig::default();
            owners.sort_unstable();
            owners.dedup();
            ensure_requirement_is_valid(owners.len() as u32, requirement);

            for owner in &owners {
                contract.is_owner.insert(owner, &());
            }

            contract.owners = owners;
            contract.transaction_list = Default::default();
            contract.requirement = requirement;
            contract
        }

        /// Add a new owner to the contract.
        ///
        /// Only callable by the wallet itself.
        ///
        /// # Panics
        ///
        /// If the owner already exists.
        ///
        /// # Examples
        ///
        /// Since this message must be send by the wallet itself it has to be build as a
        /// `Transaction` and dispatched through `submit_transaction` and
        /// `invoke_transaction`:
        /// ```should_panic
        /// use ink::{
        ///     env::{
        ///         DefaultEnvironment as Env,
        ///         Environment,
        ///         call::{
        ///             Call,
        ///             CallParams,
        ///             ExecutionInput,
        ///             Selector,
        ///             utils::ArgumentList,
        ///         },
        ///     },
        ///     scale::Encode,
        ///     selector_bytes,
        /// };
        /// use multisig::{
        ///     ConfirmationStatus,
        ///     Transaction,
        /// };
        ///
        /// // address of an existing `Multisig` contract
        /// let wallet_id: ink::Address = [7u8; 20].into();
        ///
        /// // first create the transaction that adds `alice` through `add_owner`
        /// let alice: ink::Address = [1u8; 20].into();
        /// let add_owner_args = ArgumentList::empty().push_arg(&alice);
        ///
        /// let transaction_candidate = Transaction {
        ///     callee: wallet_id,
        ///     selector: selector_bytes!("add_owner"),
        ///     input: add_owner_args.encode(),
        ///     transferred_value: ink::U256::zero(),
        ///     ref_time_limit: 0,
        ///     allow_reentry: true,
        /// };
        ///
        /// // Submit the transaction for confirmation
        /// //
        /// // Note that the selector bytes of the `submit_transaction` method
        /// // are `[86, 244, 13, 223]`.
        /// let (id, _status) = ink::env::call::build_call::<Env>()
        ///     .call_type(Call::new(wallet_id))
        ///     .ref_time_limit(0)
        ///     .exec_input(
        ///         ExecutionInput::new(Selector::new([86, 244, 13, 223]))
        ///             .push_arg(&transaction_candidate),
        ///     )
        ///     .returns::<(u32, ConfirmationStatus)>()
        ///     .invoke();
        ///
        /// // Wait until all owners have confirmed and then execute the tx.
        /// //
        /// // Note that the selector bytes of the `invoke_transaction` method
        /// // are `[185, 50, 225, 236]`.
        /// ink::env::call::build_call::<Env>()
        ///     .call_type(Call::new(wallet_id))
        ///     .ref_time_limit(0)
        ///     .exec_input(ExecutionInput::new(Selector::new([185, 50, 225, 236])).push_arg(&id))
        ///     .returns::<()>()
        ///     .invoke();
        /// ```
        #[ink(message)]
        pub fn add_owner(&mut self, new_owner: Address) {
            self.ensure_from_wallet();
            self.ensure_no_owner(&new_owner);
            ensure_requirement_is_valid(
                (self.owners.len() as u32).checked_add(1).unwrap(),
                self.requirement,
            );
            self.is_owner.insert(new_owner, &());
            self.owners.push(new_owner);
            self.env().emit_event(OwnerAddition { owner: new_owner });
        }

        /// Remove an owner from the contract.
        ///
        /// Only callable by the wallet itself. If by doing this the amount of owners
        /// would be smaller than the requirement it is adjusted to be exactly the
        /// number of owners.
        ///
        /// # Panics
        ///
        /// If `owner` is no owner of the wallet.
        #[ink(message)]
        pub fn remove_owner(&mut self, owner: Address) {
            self.ensure_from_wallet();
            self.ensure_owner(&owner);
            // If caller is an owner the len has to be > 0
            #[allow(clippy::arithmetic_side_effects)]
            let len = self.owners.len() as u32 - 1;
            let requirement = u32::min(len, self.requirement);
            ensure_requirement_is_valid(len, requirement);
            let owner_index = self.owner_index(&owner) as usize;
            self.owners.swap_remove(owner_index);
            self.is_owner.remove(owner);
            self.requirement = requirement;
            self.clean_owner_confirmations(&owner);
            self.env().emit_event(OwnerRemoval { owner });
        }

        /// Replace an owner from the contract with a new one.
        ///
        /// Only callable by the wallet itself.
        ///
        /// # Panics
        ///
        /// If `old_owner` is no owner or if `new_owner` already is one.
        #[ink(message)]
        pub fn replace_owner(&mut self, old_owner: Address, new_owner: Address) {
            self.ensure_from_wallet();
            self.ensure_owner(&old_owner);
            self.ensure_no_owner(&new_owner);
            let owner_index = self.owner_index(&old_owner);
            self.owners[owner_index as usize] = new_owner;
            self.is_owner.remove(old_owner);
            self.is_owner.insert(new_owner, &());
            self.clean_owner_confirmations(&old_owner);
            self.env().emit_event(OwnerRemoval { owner: old_owner });
            self.env().emit_event(OwnerAddition { owner: new_owner });
        }

        /// Change the requirement to a new value.
        ///
        /// Only callable by the wallet itself.
        ///
        /// # Panics
        ///
        /// If the `new_requirement` violates our invariant.
        #[ink(message)]
        pub fn change_requirement(&mut self, new_requirement: u32) {
            self.ensure_from_wallet();
            ensure_requirement_is_valid(self.owners.len() as u32, new_requirement);
            self.requirement = new_requirement;
            self.env().emit_event(RequirementChange { new_requirement });
        }

        /// Add a new transaction candidate to the contract.
        ///
        /// This also confirms the transaction for the caller. This can be called by any
        /// owner.
        #[ink(message)]
        pub fn submit_transaction(
            &mut self,
            transaction: Transaction,
        ) -> (TransactionId, ConfirmationStatus) {
            self.ensure_caller_is_owner();
            let trans_id = self.transaction_list.next_id;
            self.transaction_list.next_id =
                trans_id.checked_add(1).expect("Transaction ids exhausted.");
            self.transactions.insert(trans_id, &transaction);
            self.transaction_list.transactions.push(trans_id);
            self.env().emit_event(Submission {
                transaction: trans_id,
            });
            (
                trans_id,
                self.confirm_by_caller(self.env().caller(), trans_id),
            )
        }

        /// Remove a transaction from the contract.
        /// Only callable by the wallet itself.
        ///
        /// # Panics
        ///
        /// If `trans_id` is no valid transaction id.
        #[ink(message)]
        pub fn cancel_transaction(&mut self, trans_id: TransactionId) {
            self.ensure_from_wallet();
            if self.take_transaction(trans_id).is_some() {
                self.env().emit_event(Cancellation {
                    transaction: trans_id,
                });
            }
        }

        /// Confirm a transaction for the sender that was submitted by any owner.
        ///
        /// This can be called by any owner.
        ///
        /// # Panics
        ///
        /// If `trans_id` is no valid transaction id.
        #[ink(message)]
        pub fn confirm_transaction(
            &mut self,
            trans_id: TransactionId,
        ) -> ConfirmationStatus {
            self.ensure_caller_is_owner();
            self.ensure_transaction_exists(trans_id);
            self.confirm_by_caller(self.env().caller(), trans_id)
        }

        /// Revoke the senders confirmation.
        ///
        /// This can be called by any owner.
        ///
        /// # Panics
        ///
        /// If `trans_id` is no valid transaction id.
        #[ink(message)]
        pub fn revoke_confirmation(&mut self, trans_id: TransactionId) {
            self.ensure_caller_is_owner();
            let caller = self.env().caller();
            if self.confirmations.contains((trans_id, caller)) {
                self.confirmations.remove((trans_id, caller));
                let mut confirmation_count = self
                    .confirmation_count
                    .get(trans_id)
                    .expect(
                    "There is a entry in `self.confirmations`. Hence a count must exit.",
                );
                // Will not underflow as there is at least one confirmation
                #[allow(clippy::arithmetic_side_effects)]
                {
                    confirmation_count -= 1;
                }
                self.confirmation_count
                    .insert(trans_id, &confirmation_count);
                self.env().emit_event(Revocation {
                    transaction: trans_id,
                    from: caller,
                });
            }
        }

        /// Invoke a confirmed execution without getting its output.
        ///
        /// If the transaction which is invoked transfers value, this value has
        /// to be sent as payment with this call. The method will fail otherwise,
        /// and the transaction would then be reverted.
        ///
        /// Its return value indicates whether the called transaction was successful.
        /// This can be called by anyone.
        #[ink(message, payable)]
        pub fn invoke_transaction(
            &mut self,
            trans_id: TransactionId,
        ) -> Result<(), Error> {
            self.ensure_confirmed(trans_id);
            let t = self.take_transaction(trans_id).expect(WRONG_TRANSACTION_ID);
            assert!(self.env().transferred_value() == t.transferred_value);
            let call_flags = if t.allow_reentry {
                CallFlags::ALLOW_REENTRY
            } else {
                CallFlags::empty()
            };

            let result = build_call::<<Self as ::ink::env::ContractEnv>::Env>()
                .call(t.callee)
                .ref_time_limit(t.ref_time_limit)
                .transferred_value(t.transferred_value)
                .call_flags(call_flags)
                .exec_input(
                    ExecutionInput::new(t.selector.into()).push_arg(CallInput(&t.input)),
                )
                .returns::<()>()
                .try_invoke();

            let result = match result {
                Ok(Ok(_)) => Ok(()),
                _ => Err(Error::TransactionFailed),
            };

            self.env().emit_event(Execution {
                transaction: trans_id,
                result: result.map(|_| None),
            });
            result
        }

        /// Evaluate a confirmed execution and return its output as bytes.
        ///
        /// Its return value indicates whether the called transaction was successful and
        /// contains its output when successful.
        /// This can be called by anyone.
        #[ink(message, payable)]
        pub fn eval_transaction(
            &mut self,
            trans_id: TransactionId,
        ) -> Result<Vec<u8>, Error> {
            self.ensure_confirmed(trans_id);
            let t = self.take_transaction(trans_id).expect(WRONG_TRANSACTION_ID);
            let call_flags = if t.allow_reentry {
                CallFlags::ALLOW_REENTRY
            } else {
                CallFlags::empty()
            };

            let result = build_call::<<Self as ::ink::env::ContractEnv>::Env>()
                .call(t.callee)
                .ref_time_limit(t.ref_time_limit)
                .transferred_value(t.transferred_value)
                .call_flags(call_flags)
                .exec_input(
                    ExecutionInput::new(t.selector.into()).push_arg(CallInput(&t.input)),
                )
                .returns::<Vec<u8>>()
                .try_invoke();

            let result = match result {
                Ok(Ok(v)) => Ok(v),
                _ => Err(Error::TransactionFailed),
            };

            self.env().emit_event(Execution {
                transaction: trans_id,
                result: result.clone().map(Some),
            });
            result
        }

        /// Set the `transaction` as confirmed by `confirmer`.
        /// Idempotent operation regarding an already confirmed `transaction`
        /// by `confirmer`.
        fn confirm_by_caller(
            &mut self,
            confirmer: Address,
            transaction: TransactionId,
        ) -> ConfirmationStatus {
            let mut count = self.confirmation_count.get(transaction).unwrap_or(0);
            let key = (transaction, confirmer);
            let new_confirmation = !self.confirmations.contains(key);
            if new_confirmation {
                count = count.checked_add(1).unwrap();
                self.confirmations.insert(key, &());
                self.confirmation_count.insert(transaction, &count);
            }
            let status = {
                if count >= self.requirement {
                    ConfirmationStatus::Confirmed
                } else {
                    // We checked that count < self.requirement
                    #[allow(clippy::arithmetic_side_effects)]
                    ConfirmationStatus::ConfirmationsNeeded(self.requirement - count)
                }
            };
            if new_confirmation {
                self.env().emit_event(Confirmation {
                    transaction,
                    from: confirmer,
                    status,
                });
            }
            status
        }

        /// Get the index of `owner` in `self.owners`.
        /// Panics if `owner` is not found in `self.owners`.
        fn owner_index(&self, owner: &Address) -> u32 {
            self.owners.iter().position(|x| *x == *owner).expect(
                "This is only called after it was already verified that the id is
                 actually an owner.",
            ) as u32
        }

        /// Remove the transaction identified by `trans_id` from `self.transactions`.
        /// Also removes all confirmation state associated with it.
        fn take_transaction(&mut self, trans_id: TransactionId) -> Option<Transaction> {
            let transaction = self.transactions.get(trans_id);
            if transaction.is_some() {
                self.transactions.remove(trans_id);
                let pos = self
                    .transaction_list
                    .transactions
                    .iter()
                    .position(|t| t == &trans_id)
                    .expect("The transaction exists hence it must also be in the list.");
                self.transaction_list.transactions.swap_remove(pos);
                for owner in self.owners.iter() {
                    self.confirmations.remove((trans_id, *owner));
                }
                self.confirmation_count.remove(trans_id);
            }
            transaction
        }

        /// Remove all confirmation state associated with `owner`.
        /// Also adjusts the `self.confirmation_count` variable.
        fn clean_owner_confirmations(&mut self, owner: &Address) {
            for trans_id in &self.transaction_list.transactions {
                let key = (*trans_id, *owner);
                if self.confirmations.contains(key) {
                    self.confirmations.remove(key);
                    let mut count = self.confirmation_count.get(trans_id).unwrap_or(0);
                    count = count.saturating_sub(1);
                    self.confirmation_count.insert(trans_id, &count);
                }
            }
        }

        /// Panic if transaction `trans_id` is not confirmed by at least
        /// `self.requirement` owners.
        fn ensure_confirmed(&self, trans_id: TransactionId) {
            assert!(
                self.confirmation_count
                    .get(trans_id)
                    .expect(WRONG_TRANSACTION_ID)
                    >= self.requirement
            );
        }

        /// Panic if the transaction `trans_id` does not exit.
        fn ensure_transaction_exists(&self, trans_id: TransactionId) {
            self.transactions.get(trans_id).expect(WRONG_TRANSACTION_ID);
        }

        /// Panic if the sender is no owner of the wallet.
        fn ensure_caller_is_owner(&self) {
            self.ensure_owner(&self.env().caller());
        }

        /// Panic if the sender is not this wallet.
        fn ensure_from_wallet(&self) {
            assert_eq!(self.env().caller(), self.env().address());
        }

        /// Panic if `owner` is not an owner,
        fn ensure_owner(&self, owner: &Address) {
            assert!(self.is_owner.contains(owner));
        }

        /// Panic if `owner` is an owner.
        fn ensure_no_owner(&self, owner: &Address) {
            assert!(!self.is_owner.contains(owner));
        }
    }

    /// Panic if the number of `owners` under a `requirement` violates our
    /// requirement invariant.
    fn ensure_requirement_is_valid(owners: u32, requirement: u32) {
        assert!(0 < requirement && requirement <= owners && owners <= MAX_OWNERS);
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            call::utils::ArgumentList,
            test,
        };

        const WALLET: [u8; 20] = [7; 20];

        impl Transaction {
            fn change_requirement(requirement: u32) -> Self {
                use ink::scale::Encode;
                let call_args = ArgumentList::empty().push_arg(&requirement);

                // Multisig::change_requirement()
                Self {
                    callee: Address::from(WALLET),
                    selector: ink::selector_bytes!("change_requirement"),
                    input: call_args.encode(),
                    transferred_value: U256::zero(),
                    ref_time_limit: 1000000,
                    allow_reentry: false,
                }
            }
        }

        fn set_caller(sender: Address) {
            ink::env::test::set_caller(sender);
        }

        fn set_from_wallet() {
            let callee = Address::from(WALLET);
            set_caller(callee);
        }

        fn set_from_owner() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
        }

        fn set_from_no_owner() {
            let accounts = default_accounts();
            set_caller(accounts.django);
        }

        fn default_accounts() -> test::DefaultAccounts {
            ink::env::test::default_accounts()
        }

        fn build_contract() -> Multisig {
            // Set the contract's address as `WALLET`.
            let callee: Address = Address::from(WALLET);
            ink::env::test::set_callee(callee);

            let accounts = default_accounts();
            let owners = vec![accounts.alice, accounts.bob, accounts.eve];
            Multisig::new(2, owners)
        }

        fn submit_transaction() -> Multisig {
            let mut contract = build_contract();
            let accounts = default_accounts();
            set_from_owner();
            contract.submit_transaction(Transaction::change_requirement(1));
            assert_eq!(contract.transaction_list.transactions.len(), 1);
            assert_eq!(test::recorded_events().len(), 2);
            let transaction = contract.transactions.get(0).unwrap();
            assert_eq!(transaction, Transaction::change_requirement(1));
            contract.confirmations.get((0, accounts.alice)).unwrap();
            assert_eq!(contract.confirmation_count.get(0).unwrap(), 1);
            contract
        }

        #[ink::test]
        fn construction_works() {
            let accounts = default_accounts();
            let owners = [accounts.alice, accounts.bob, accounts.eve];
            let contract = build_contract();

            assert_eq!(contract.owners.len(), 3);
            assert_eq!(contract.requirement, 2);
            use ink::prelude::collections::HashSet;
            assert_eq!(
                HashSet::<&Address>::from_iter(contract.owners.iter()),
                HashSet::from_iter(owners.iter()),
            );
            assert!(contract.is_owner.contains(accounts.alice));
            assert!(contract.is_owner.contains(accounts.bob));
            assert!(contract.is_owner.contains(accounts.eve));
            assert!(!contract.is_owner.contains(accounts.charlie));
            assert!(!contract.is_owner.contains(accounts.django));
            assert!(!contract.is_owner.contains(accounts.frank));
            assert_eq!(contract.transaction_list.transactions.len(), 0);
        }

        #[ink::test]
        #[should_panic]
        fn empty_owner_construction_fails() {
            Multisig::new(0, vec![]);
        }

        #[ink::test]
        #[should_panic]
        fn zero_requirement_construction_fails() {
            let accounts = default_accounts();
            Multisig::new(0, vec![accounts.alice, accounts.bob]);
        }

        #[ink::test]
        #[should_panic]
        fn too_large_requirement_construction_fails() {
            let accounts = default_accounts();
            Multisig::new(3, vec![accounts.alice, accounts.bob]);
        }

        #[ink::test]
        fn add_owner_works() {
            let accounts = default_accounts();
            let mut contract = build_contract();
            set_from_wallet();
            let owners = contract.owners.len();
            contract.add_owner(accounts.frank);
            assert_eq!(contract.owners.len(), owners + 1);
            assert!(contract.is_owner.contains(accounts.frank));
            assert_eq!(test::recorded_events().len(), 1);
        }

        #[ink::test]
        #[should_panic]
        fn add_existing_owner_fails() {
            let accounts = default_accounts();
            let mut contract = build_contract();
            set_from_wallet();
            contract.add_owner(accounts.bob);
        }

        #[ink::test]
        #[should_panic]
        fn add_owner_permission_denied() {
            let accounts = default_accounts();
            let mut contract = build_contract();
            set_from_owner();
            contract.add_owner(accounts.frank);
        }

        #[ink::test]
        fn remove_owner_works() {
            let accounts = default_accounts();
            let mut contract = build_contract();
            set_from_wallet();
            let owners = contract.owners.len();
            contract.remove_owner(accounts.alice);
            assert_eq!(contract.owners.len(), owners - 1);
            assert!(!contract.is_owner.contains(accounts.alice));
            assert_eq!(test::recorded_events().len(), 1);
        }

        #[ink::test]
        #[should_panic]
        fn remove_owner_nonexisting_fails() {
            let accounts = default_accounts();
            let mut contract = build_contract();
            set_from_wallet();
            contract.remove_owner(accounts.django);
        }

        #[ink::test]
        #[should_panic]
        fn remove_owner_permission_denied() {
            let accounts = default_accounts();
            let mut contract = build_contract();
            set_from_owner();
            contract.remove_owner(accounts.alice);
        }

        #[ink::test]
        fn replace_owner_works() {
            let accounts = default_accounts();
            let mut contract = build_contract();
            set_from_wallet();
            let owners = contract.owners.len();
            contract.replace_owner(accounts.alice, accounts.django);
            assert_eq!(contract.owners.len(), owners);
            assert!(!contract.is_owner.contains(accounts.alice));
            assert!(contract.is_owner.contains(accounts.django));
            assert_eq!(test::recorded_events().len(), 2);
        }

        #[ink::test]
        #[should_panic]
        fn replace_owner_existing_fails() {
            let accounts = default_accounts();
            let mut contract = build_contract();
            set_from_wallet();
            contract.replace_owner(accounts.alice, accounts.bob);
        }

        #[ink::test]
        #[should_panic]
        fn replace_owner_nonexisting_fails() {
            let accounts = default_accounts();
            let mut contract = build_contract();
            set_from_wallet();
            contract.replace_owner(accounts.django, accounts.frank);
        }

        #[ink::test]
        #[should_panic]
        fn replace_owner_permission_denied() {
            let accounts = default_accounts();
            let mut contract = build_contract();
            set_from_owner();
            contract.replace_owner(accounts.alice, accounts.django);
        }

        #[ink::test]
        fn change_requirement_works() {
            let mut contract = build_contract();
            assert_eq!(contract.requirement, 2);
            set_from_wallet();
            contract.change_requirement(3);
            assert_eq!(contract.requirement, 3);
            assert_eq!(test::recorded_events().len(), 1);
        }

        #[ink::test]
        #[should_panic]
        fn change_requirement_too_high() {
            let mut contract = build_contract();
            set_from_wallet();
            contract.change_requirement(4);
        }

        #[ink::test]
        #[should_panic]
        fn change_requirement_zero_fails() {
            let mut contract = build_contract();
            set_from_wallet();
            contract.change_requirement(0);
        }

        #[ink::test]
        fn submit_transaction_works() {
            submit_transaction();
        }

        #[ink::test]
        #[should_panic]
        fn submit_transaction_no_owner_fails() {
            let mut contract = build_contract();
            set_from_no_owner();
            contract.submit_transaction(Transaction::change_requirement(1));
        }

        #[ink::test]
        #[should_panic]
        fn submit_transaction_wallet_fails() {
            let mut contract = build_contract();
            set_from_wallet();
            contract.submit_transaction(Transaction::change_requirement(1));
        }

        #[ink::test]
        fn cancel_transaction_works() {
            let mut contract = submit_transaction();
            set_from_wallet();
            contract.cancel_transaction(0);
            assert_eq!(contract.transaction_list.transactions.len(), 0);
            assert_eq!(test::recorded_events().len(), 3);
        }

        #[ink::test]
        fn cancel_transaction_nonexisting() {
            let mut contract = submit_transaction();
            set_from_wallet();
            contract.cancel_transaction(1);
            assert_eq!(contract.transaction_list.transactions.len(), 1);
            assert_eq!(test::recorded_events().len(), 2);
        }

        #[ink::test]
        #[should_panic]
        fn cancel_transaction_no_permission() {
            let mut contract = submit_transaction();
            contract.cancel_transaction(0);
        }

        #[ink::test]
        fn confirm_transaction_works() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_caller(accounts.bob);
            contract.confirm_transaction(0);
            assert_eq!(test::recorded_events().len(), 3);
            contract.confirmations.get((0, accounts.bob)).unwrap();
            assert_eq!(contract.confirmation_count.get(0).unwrap(), 2);
        }

        #[ink::test]
        fn revoke_confirmations() {
            // given
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            // Confirm by Bob
            set_caller(accounts.bob);
            contract.confirm_transaction(0);
            // Confirm by Eve
            set_caller(accounts.eve);
            contract.confirm_transaction(0);
            assert_eq!(contract.confirmation_count.get(0).unwrap(), 3);
            // Revoke from Eve
            contract.revoke_confirmation(0);
            assert_eq!(contract.confirmation_count.get(0).unwrap(), 2);
            // Revoke from Bob
            set_caller(accounts.bob);
            contract.revoke_confirmation(0);
            assert_eq!(contract.confirmation_count.get(0).unwrap(), 1);
        }

        #[ink::test]
        fn confirm_transaction_already_confirmed() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_caller(accounts.alice);
            contract.confirm_transaction(0);
            assert_eq!(test::recorded_events().len(), 2);
            contract.confirmations.get((0, accounts.alice)).unwrap();
            assert_eq!(contract.confirmation_count.get(0).unwrap(), 1);
        }

        #[ink::test]
        #[should_panic]
        fn confirm_transaction_no_owner_fail() {
            let mut contract = submit_transaction();
            set_from_no_owner();
            contract.confirm_transaction(0);
        }

        #[ink::test]
        fn revoke_transaction_works() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_caller(accounts.alice);
            contract.revoke_confirmation(0);
            assert_eq!(test::recorded_events().len(), 3);
            assert!(!contract.confirmations.contains((0, accounts.alice)));
            assert_eq!(contract.confirmation_count.get(0).unwrap(), 0);
        }

        #[ink::test]
        fn revoke_transaction_no_confirmer() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_caller(accounts.bob);
            contract.revoke_confirmation(0);
            assert_eq!(test::recorded_events().len(), 2);
            assert!(contract.confirmations.contains((0, accounts.alice)));
            assert_eq!(contract.confirmation_count.get(0).unwrap(), 1);
        }

        #[ink::test]
        #[should_panic]
        fn revoke_transaction_no_owner_fail() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_caller(accounts.django);
            contract.revoke_confirmation(0);
        }

        #[ink::test]
        fn execute_transaction_works() {
            // Execution of calls is currently unsupported in off-chain test.
            // Calling `execute_transaction` panics in any case.
        }
    }
}
