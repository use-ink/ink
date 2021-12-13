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

#![cfg_attr(not(feature = "std"), no_std)]

pub use self::multisig::{
    ConfirmationStatus,
    Multisig,
    Transaction,
};
use ink_lang as ink;

#[ink::contract]
mod multisig {
    use ink_env::call::{
        build_call,
        utils::ReturnType,
        ExecutionInput,
    };
    use ink_prelude::vec::Vec;
    use ink_storage::{
        lazy::Mapping,
        traits::{
            PackedLayout,
            SpreadAllocate,
            SpreadLayout,
        },
        Lazy,
    };
    use scale::Output;

    /// Tune this to your liking but be wary that allowing too many owners will not perform well.
    const MAX_OWNERS: u32 = 50;

    type TransactionId = u32;
    const WRONG_TRANSACTION_ID: &str =
        "The user specified an invalid transaction id. Abort.";

    /// A wrapper that allows us to encode a blob of bytes.
    ///
    /// We use this to pass the set of untyped (bytes) parameters to the `CallBuilder`.
    struct CallInput<'a>(&'a [u8]);

    impl<'a> scale::Encode for CallInput<'a> {
        fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
            dest.write(self.0);
        }
    }

    /// Indicates whether a transaction is already confirmed or needs further confirmations.
    #[derive(scale::Encode, scale::Decode, Clone, Copy, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub enum ConfirmationStatus {
        /// The transaction is already confirmed.
        Confirmed,
        /// Indicates how many confirmations are remaining.
        ConfirmationsNeeded(u32),
    }

    /// A Transaction is what every `owner` can submit for confirmation by other owners.
    /// If enough owners agree it will be executed by the contract.
    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink_storage::traits::StorageLayout
        )
    )]
    pub struct Transaction {
        /// The `AccountId` of the contract that is called in this transaction.
        pub callee: AccountId,
        /// The selector bytes that identifies the function of the callee that should be called.
        pub selector: [u8; 4],
        /// The SCALE encoded parameters that are passed to the called function.
        pub input: Vec<u8>,
        /// The amount of chain balance that is transferred to the callee.
        pub transferred_value: Balance,
        /// Gas limit for the execution of the call.
        pub gas_limit: u64,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if the call failed.
        TransactionFailed,
    }

    /// This is a book keeping struct that stores a list of all transaction ids and
    /// also the next id to use. We need it for cleaning up the storage.
    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout, Default)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink_storage::traits::StorageLayout
        )
    )]
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
        from: AccountId,
        /// The confirmation status after this confirmation was applied.
        #[ink(topic)]
        status: ConfirmationStatus,
    }

    /// Emitted when an owner revoked a confirmation.
    #[ink(event)]
    pub struct Revokation {
        /// The transaction that was revoked.
        #[ink(topic)]
        transaction: TransactionId,
        /// The owner that sent the revocation.
        #[ink(topic)]
        from: AccountId,
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
    pub struct Cancelation {
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
        /// Indicates whether the transaction executed successfully. If so the `Ok` value holds
        /// the output in bytes. The Option is `None` when the transaction was executed through
        /// `invoke_transaction` rather than `evaluate_transaction`.
        #[ink(topic)]
        result: Result<Option<Vec<u8>>, Error>,
    }

    /// Emitted when an owner is added to the wallet.
    #[ink(event)]
    pub struct OwnerAddition {
        /// The owner that was added.
        #[ink(topic)]
        owner: AccountId,
    }

    /// Emitted when an owner is removed from the wallet.
    #[ink(event)]
    pub struct OwnerRemoval {
        /// The owner that was removed.
        #[ink(topic)]
        owner: AccountId,
    }

    /// Emitted when the requirement changed.
    #[ink(event)]
    pub struct RequirementChange {
        /// The new requirement value.
        new_requirement: u32,
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Multisig {
        /// Every entry in this map represents the confirmation of an owner for a
        /// transaction. This is effectively a set rather than a map.
        confirmations: Mapping<(TransactionId, AccountId), ()>,
        /// The amount of confirmations for every transaction. This is a redundant
        /// information and is kept in order to prevent iterating through the
        /// confirmation set to check if a transaction is confirmed.
        confirmation_count: Mapping<TransactionId, u32>,
        /// Map the transaction id to its unexecuted transaction.
        transactions: Mapping<TransactionId, Transaction>,
        /// We need to hold a list of all transactions so that we can clean up storage
        /// when an owner is removed.
        transaction_list: Lazy<Transactions>,
        /// The list is a vector because iterating over it is necessary when cleaning
        /// up the confirmation set.
        owners: Lazy<Vec<AccountId>>,
        /// Redundant information to speed up the check whether a caller is an owner.
        is_owner: Mapping<AccountId, ()>,
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
        pub fn new(requirement: u32, mut owners: Vec<AccountId>) -> Self {
            ink_lang::codegen::initialize_contract(|contract: &mut Self| {
                owners.sort_unstable();
                owners.dedup();
                ensure_requirement_is_valid(owners.len() as u32, requirement);

                for owner in &owners {
                    contract.is_owner.insert(owner, &());
                }

                contract.owners = owners.into();
                contract.transaction_list = Default::default();
                contract.requirement = requirement.into();
            })
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
        /// `Transaction` and dispatched through `submit_transaction` and `invoke_transaction`:
        /// ```no_run
        /// use ink_env::{DefaultEnvironment as Env, AccountId, call::{CallParams, Selector}, test::CallData};
        /// use multisig::{Transaction, ConfirmationStatus};
        ///
        /// // address of an existing `Multisig` contract
        /// let wallet_id: AccountId = [7u8; 32].into();
        ///
        /// // first create the transaction that adds `alice` through `add_owner`
        /// let alice: AccountId = [1u8; 32].into();
        /// let mut call = CallData::new(Selector::new([166, 229, 27, 154])); // add_owner
        /// call.push_arg(&alice);
        /// let transaction = Transaction {
        ///     callee: wallet_id,
        ///     selector: call.selector().to_bytes(),
        ///     input: call.params().to_owned(),
        ///     transferred_value: 0,
        ///     gas_limit: 0
        /// };
        ///
        /// // submit the transaction for confirmation
        /// let mut submit = CallParams::<Env, _, _>::eval(
        ///     wallet_id,
        ///     Selector::new([86, 244, 13, 223]) // submit_transaction
        /// );
        /// let (id, _): (u32, ConfirmationStatus)  = submit.push_arg(&transaction)
        ///     .fire()
        ///     .expect("submit_transaction won't panic.");
        ///
        /// // wait until all required owners have confirmed and then execute the transaction
        /// let mut invoke = CallParams::<Env, _, ()>::invoke(
        ///     wallet_id,
        ///     Selector::new([185, 50, 225, 236]) // invoke_transaction
        /// );
        /// invoke.push_arg(&id).fire();
        /// ```
        #[ink(message)]
        pub fn add_owner(&mut self, new_owner: AccountId) {
            self.ensure_from_wallet();
            self.ensure_no_owner(&new_owner);
            ensure_requirement_is_valid(self.owners.len() as u32 + 1, self.requirement);
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
        pub fn remove_owner(&mut self, owner: AccountId) {
            self.ensure_from_wallet();
            self.ensure_owner(&owner);
            let len = self.owners.len() as u32 - 1;
            let requirement = u32::min(len, self.requirement);
            ensure_requirement_is_valid(len, requirement);
            let owner_index = self.owner_index(&owner) as usize;
            self.owners.swap_remove(owner_index);
            self.is_owner.remove(&owner);
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
        pub fn replace_owner(&mut self, old_owner: AccountId, new_owner: AccountId) {
            self.ensure_from_wallet();
            self.ensure_owner(&old_owner);
            self.ensure_no_owner(&new_owner);
            let owner_index = self.owner_index(&old_owner);
            self.owners[owner_index as usize] = new_owner;
            self.is_owner.remove(&old_owner);
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
        /// This also confirms the transaction for the caller. This can be called by any owner.
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
                self.env().emit_event(Cancelation {
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
            if self.confirmations.get(&(trans_id, caller)).is_some() {
                self.confirmations.remove(&(trans_id, caller));
                let mut confirmation_count = self
                    .confirmation_count
                    .get(&trans_id)
                    .expect(
                    "There is a entry in `self.confirmations`. Hence a count must exit.",
                );
                // Will not underflow as there is at least one confirmation
                confirmation_count -= 1;
                self.confirmation_count
                    .insert(&trans_id, &confirmation_count);
                self.env().emit_event(Revokation {
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
            let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                .callee(t.callee)
                .gas_limit(t.gas_limit)
                .transferred_value(t.transferred_value)
                .exec_input(
                    ExecutionInput::new(t.selector.into()).push_arg(CallInput(&t.input)),
                )
                .returns::<()>()
                .fire()
                .map_err(|_| Error::TransactionFailed);
            self.env().emit_event(Execution {
                transaction: trans_id,
                result: result.map(|_| None),
            });
            result
        }

        /// Evaluate a confirmed execution and return its output as bytes.
        ///
        /// Its return value indicates whether the called transaction was successful and contains
        /// its output when successful.
        /// This can be called by anyone.
        #[ink(message, payable)]
        pub fn eval_transaction(
            &mut self,
            trans_id: TransactionId,
        ) -> Result<Vec<u8>, Error> {
            self.ensure_confirmed(trans_id);
            let t = self.take_transaction(trans_id).expect(WRONG_TRANSACTION_ID);
            let result = build_call::<<Self as ::ink_lang::reflect::ContractEnv>::Env>()
                .callee(t.callee)
                .gas_limit(t.gas_limit)
                .transferred_value(t.transferred_value)
                .exec_input(
                    ExecutionInput::new(t.selector.into()).push_arg(CallInput(&t.input)),
                )
                .returns::<ReturnType<Vec<u8>>>()
                .fire()
                .map_err(|_| Error::TransactionFailed);
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
            confirmer: AccountId,
            transaction: TransactionId,
        ) -> ConfirmationStatus {
            let mut count = self.confirmation_count.get(&transaction).unwrap_or(0);
            let key = (transaction, confirmer);
            let new_confirmation = self.confirmations.get(&key).is_none();
            if new_confirmation {
                count += 1;
                self.confirmations.insert(&key, &());
                self.confirmation_count.insert(&transaction, &count);
            }
            let status = {
                if count >= self.requirement {
                    ConfirmationStatus::Confirmed
                } else {
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
        fn owner_index(&self, owner: &AccountId) -> u32 {
            self.owners.iter().position(|x| *x == *owner).expect(
                "This is only called after it was already verified that the id is
                 actually an owner.",
            ) as u32
        }

        /// Remove the transaction identified by `trans_id` from `self.transactions`.
        /// Also removes all confirmation state associated with it.
        fn take_transaction(&mut self, trans_id: TransactionId) -> Option<Transaction> {
            let transaction = self.transactions.get(&trans_id);
            if transaction.is_some() {
                self.transactions.remove(&trans_id);
                let pos = self
                    .transaction_list
                    .transactions
                    .iter()
                    .position(|t| t == &trans_id)
                    .expect("The transaction exists hence it must also be in the list.");
                self.transaction_list.transactions.swap_remove(pos);
                for owner in self.owners.iter() {
                    self.confirmations.remove(&(trans_id, *owner));
                }
                self.confirmation_count.remove(&trans_id);
            }
            transaction
        }

        /// Remove all confirmation state associated with `owner`.
        /// Also adjusts the `self.confirmation_count` variable.
        fn clean_owner_confirmations(&mut self, owner: &AccountId) {
            for trans_id in &self.transaction_list.transactions {
                let key = (*trans_id, *owner);
                if self.confirmations.get(&key).is_some() {
                    self.confirmations.remove(&key);
                    let mut count = self.confirmation_count.get(&trans_id).unwrap_or(0);
                    count -= 1;
                    self.confirmation_count.insert(&trans_id, &count);
                }
            }
        }

        /// Panic if transaction `trans_id` is not confirmed by at least
        /// `self.requirement` owners.
        fn ensure_confirmed(&self, trans_id: TransactionId) {
            assert!(
                self.confirmation_count
                    .get(&trans_id)
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
            assert_eq!(self.env().caller(), self.env().account_id());
        }

        /// Panic if `owner` is not an owner,
        fn ensure_owner(&self, owner: &AccountId) {
            assert!(self.is_owner.get(owner).is_some());
        }

        /// Panic if `owner` is an owner.
        fn ensure_no_owner(&self, owner: &AccountId) {
            assert!(self.is_owner.get(owner).is_none());
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
        use ink_env::{
            call,
            test,
        };
        use ink_lang as ink;
        type Accounts = test::DefaultAccounts<Environment>;
        const WALLET: [u8; 32] = [7; 32];

        impl Transaction {
            fn change_requirement(requirement: u32) -> Self {
                let mut call = test::CallData::new(call::Selector::new([0x00; 4])); // change_requirement
                call.push_arg(&requirement);
                Self {
                    callee: WALLET.into(),
                    selector: call.selector().to_bytes(),
                    input: call.params().to_owned(),
                    transferred_value: 0,
                    gas_limit: 1000000,
                }
            }
        }

        fn set_sender(sender: AccountId) {
            test::push_execution_context::<Environment>(
                sender,
                WALLET.into(),
                1000000,
                1000000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
        }

        fn set_from_wallet() {
            set_sender(WALLET.into());
        }

        fn set_from_owner() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
        }

        fn set_from_noowner() {
            let accounts = default_accounts();
            set_sender(accounts.django);
        }

        fn default_accounts() -> Accounts {
            test::default_accounts()
                .expect("Test environment is expected to be initialized.")
        }

        fn build_contract() -> Multisig {
            let accounts = default_accounts();
            let owners = ink_prelude::vec![accounts.alice, accounts.bob, accounts.eve];
            Multisig::new(2, owners)
        }

        fn submit_transaction() -> Multisig {
            let mut contract = build_contract();
            let accounts = default_accounts();
            set_from_owner();
            contract.submit_transaction(Transaction::change_requirement(1));
            assert_eq!(contract.transaction_list.transactions.len(), 1);
            assert_eq!(test::recorded_events().count(), 2);
            let transaction = contract.transactions.get(0).unwrap();
            assert_eq!(transaction, Transaction::change_requirement(1));
            contract.confirmations.get(&(0, accounts.alice)).unwrap();
            assert_eq!(contract.confirmation_count.get(&0).unwrap(), 1);
            contract
        }

        #[ink::test]
        fn construction_works() {
            let accounts = default_accounts();
            let owners = ink_prelude::vec![accounts.alice, accounts.bob, accounts.eve];
            let contract = build_contract();

            assert_eq!(contract.owners.len(), 3);
            assert_eq!(contract.requirement, 2);
            assert!(contract.owners.iter().eq(owners.iter()));
            assert!(contract.is_owner.get(&accounts.alice).is_some());
            assert!(contract.is_owner.get(&accounts.bob).is_some());
            assert!(contract.is_owner.get(&accounts.eve).is_some());
            assert!(contract.is_owner.get(&accounts.charlie).is_none());
            assert!(contract.is_owner.get(&accounts.django).is_none());
            assert!(contract.is_owner.get(&accounts.frank).is_none());
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
            assert!(contract.is_owner.get(&accounts.frank).is_some());
            assert_eq!(test::recorded_events().count(), 1);
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
            assert!(contract.is_owner.get(&accounts.alice).is_none());
            assert_eq!(test::recorded_events().count(), 1);
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
            assert!(contract.is_owner.get(&accounts.alice).is_none());
            assert!(contract.is_owner.get(&accounts.django).is_some());
            assert_eq!(test::recorded_events().count(), 2);
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
            assert_eq!(test::recorded_events().count(), 1);
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
        fn submit_transaction_noowner_fails() {
            let mut contract = build_contract();
            set_from_noowner();
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
            assert_eq!(test::recorded_events().count(), 3);
        }

        #[ink::test]
        fn cancel_transaction_nonexisting() {
            let mut contract = submit_transaction();
            set_from_wallet();
            contract.cancel_transaction(1);
            assert_eq!(contract.transaction_list.transactions.len(), 1);
            assert_eq!(test::recorded_events().count(), 2);
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
            set_sender(accounts.bob);
            contract.confirm_transaction(0);
            assert_eq!(test::recorded_events().count(), 3);
            contract.confirmations.get(&(0, accounts.bob)).unwrap();
            assert_eq!(contract.confirmation_count.get(&0).unwrap(), 2);
        }

        #[ink::test]
        fn revoke_confirmations() {
            // given
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            // Confirm by Bob
            set_sender(accounts.bob);
            contract.confirm_transaction(0);
            // Confirm by Eve
            set_sender(accounts.eve);
            contract.confirm_transaction(0);
            assert_eq!(contract.confirmation_count.get(&0).unwrap(), 3);
            // Revoke from Eve
            contract.revoke_confirmation(0);
            assert_eq!(contract.confirmation_count.get(&0).unwrap(), 2);
            // Revoke from Bob
            set_sender(accounts.bob);
            contract.revoke_confirmation(0);
            assert_eq!(contract.confirmation_count.get(&0).unwrap(), 1);
        }

        #[ink::test]
        fn confirm_transaction_already_confirmed() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_sender(accounts.alice);
            contract.confirm_transaction(0);
            assert_eq!(test::recorded_events().count(), 2);
            contract.confirmations.get(&(0, accounts.alice)).unwrap();
            assert_eq!(contract.confirmation_count.get(&0).unwrap(), 1);
        }

        #[ink::test]
        #[should_panic]
        fn confirm_transaction_noowner_fail() {
            let mut contract = submit_transaction();
            set_from_noowner();
            contract.confirm_transaction(0);
        }

        #[ink::test]
        fn revoke_transaction_works() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_sender(accounts.alice);
            contract.revoke_confirmation(0);
            assert_eq!(test::recorded_events().count(), 3);
            assert!(contract.confirmations.get(&(0, accounts.alice)).is_none());
            assert_eq!(contract.confirmation_count.get(&0).unwrap(), 0);
        }

        #[ink::test]
        fn revoke_transaction_no_confirmer() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_sender(accounts.bob);
            contract.revoke_confirmation(0);
            assert_eq!(test::recorded_events().count(), 2);
            assert!(contract.confirmations.get(&(0, accounts.alice)).is_some());
            assert_eq!(contract.confirmation_count.get(&0).unwrap(), 1);
        }

        #[ink::test]
        #[should_panic]
        fn revoke_transaction_noowner_fail() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_sender(accounts.django);
            contract.revoke_confirmation(0);
        }

        #[ink::test]
        fn execute_transaction_works() {
            // Execution of calls is currently unsupported in off-chain test.
            // Calling execute_transaction panics in any case.
        }
    }
}
