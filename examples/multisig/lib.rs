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
//! the usual confirm, execute cycle as any other transaction that should be
//! called by the wallet. For example, to add an owner you would submit a transaction
//! that calls the wallets own `add_owner` message through `confirm_transaction`.
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
//! `confirm_transaction`, `cancel_transaction`,
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
    use ink_env::{
        call::{
            build_call,
            utils::ReturnType,
            ExecutionInput,
        },
        hash::{
            Blake2x256,
            HashOutput,
        },
    };
    use ink_prelude::{
        vec,
        vec::Vec,
    };
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
    const MAX_OWNERS: u32 = 10;

    type TransactionId = Hash;
    type OwnersSetHash = Hash;
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
    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub enum ConfirmationStatus {
        /// The transaction is already fully confirmed.
        FullyConfirmed(OwnersSetHash),
        /// Indicates who confirmed the transaction.
        ///
        /// # Note: It means that there are not enough confirmations.
        PartialConfirmed(OwnersSetHash, Vec<AccountId>),
        /// The transaction is canceled.
        Canceled(OwnersSetHash),
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

    /// The owner can confirm transaction by `Transaction` itself or by `TransactionId`.
    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub enum TransactionToConfirm {
        Transaction(Transaction),
        TransactionId(TransactionId),
    }

    impl TransactionToConfirm {
        fn to_transaction_id(&self) -> TransactionId {
            match self {
                TransactionToConfirm::Transaction(tx) => {
                    // Hash from the transaction is a `TransactionId`
                    let mut output = <Blake2x256 as HashOutput>::Type::default();
                    ink_env::hash_encoded::<Blake2x256, _>(&tx, &mut output);
                    output.into()
                }
                TransactionToConfirm::TransactionId(tx_id) => tx_id.clone(),
            }
        }
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if the call failed.
        TransactionFailed,
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Multisig {
        /// Map the transaction id to its unexecuted transaction.
        transactions: Mapping<TransactionId, Transaction>,
        /// Map the transaction id to the status of the transaction.
        /// `PartialConfirmed` status tracks who confirmed transaction.
        transaction_status: Mapping<TransactionId, ConfirmationStatus>,
        /// The hash of current owners set.
        owners_set_hash: OwnersSetHash,
        /// The list is a vector because it is used to calculate `self.owners_set_hash`.
        owners: Lazy<Vec<AccountId>>,
        /// Redundant information to speed up the check whether a caller is an owner.
        is_owner: Mapping<AccountId, ()>,
        /// Minimum number of owners that have to confirm a transaction to be executed.
        requirement: u32,
    }

    /// Emitted when an owner confirms a transaction.
    #[ink(event)]
    pub struct Confirmation {
        /// The transaction that was confirmed.
        #[ink(topic)]
        transaction_id: TransactionId,
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
        /// The transaction id that was revoked.
        #[ink(topic)]
        transaction_id: TransactionId,
        /// The owner that sent the revocation.
        #[ink(topic)]
        from: AccountId,
    }

    /// Emitted when an owner submits a transaction.
    #[ink(event)]
    pub struct Submission {
        /// The id of transaction that was submitted.
        #[ink(topic)]
        transaction_id: TransactionId,
    }

    /// Emitted when a transaction was canceled.
    #[ink(event)]
    pub struct Cancelation {
        /// The transaction id that was canceled.
        #[ink(topic)]
        transaction_id: TransactionId,
    }

    /// Emitted when a transaction was executed.
    #[ink(event)]
    pub struct Execution {
        /// The transaction id that was executed.
        #[ink(topic)]
        transaction_id: TransactionId,
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

                contract.owners_set_hash = owners_set_hash(&owners);
                contract.owners = owners.into();
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
        /// `Transaction` and dispatched through `confirm_transaction` and `invoke_transaction`:
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
        /// let mut confirm = CallParams::<Env, _, _>::eval(
        ///     wallet_id,
        ///     Selector::new([86, 244, 13, 223]) // confirm_transaction
        /// );
        /// let (id, _): (u32, ConfirmationStatus)  = confirm.push_arg(&transaction)
        ///     .fire()
        ///     .expect("confirm_transaction won't panic.");
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
            self.owners_set_hash = owners_set_hash(&self.owners);
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
            self.owners_set_hash = owners_set_hash(&self.owners);
            self.requirement = requirement;
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
            self.owners_set_hash = owners_set_hash(&self.owners);
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
                self.transaction_status.insert(
                    trans_id,
                    &ConfirmationStatus::Canceled(self.owners_set_hash.clone()),
                );
                self.env().emit_event(Cancelation {
                    transaction_id: trans_id,
                });
            }
        }

        /// Confirm a transaction for the sender by any owner. If it is a new transaction,
        /// it will be added to `self.transactions`.
        ///
        /// This can be called by any owner.
        ///
        /// # Panics
        ///
        /// If `trans_id` is no valid transaction id.
        #[ink(message)]
        pub fn confirm_transaction(
            &mut self,
            trans_to_confirm: TransactionToConfirm,
        ) -> ConfirmationStatus {
            self.ensure_caller_is_owner();

            let trans_id = trans_to_confirm.to_transaction_id();
            if self.transactions.get(&trans_id).is_none() {
                match trans_to_confirm {
                    TransactionToConfirm::Transaction(tx) => {
                        self.transactions.insert(trans_id, &tx);
                        // We want to override status here in case if it was canceled before or executed
                        self.transaction_status.insert(
                            trans_id,
                            &ConfirmationStatus::PartialConfirmed(
                                self.owners_set_hash,
                                vec![],
                            ),
                        );
                        self.env().emit_event(Submission {
                            transaction_id: trans_id,
                        });
                    }
                    TransactionToConfirm::TransactionId(_) => {
                        panic!("It is a new transaction you should submit a body of the transaction")
                    }
                }
            }

            // We know that it exists in PartialConfirmed or FullyConfirmed state
            let status = self.transaction_status.get(&trans_id).unwrap();

            // We only need to confirm partial transactions, if it is FullyConfirmed but in `self.transactions`
            // it means that it is not executed
            if let ConfirmationStatus::PartialConfirmed(hash, mut set) = status {
                let caller = self.env().caller();
                self.actualize_confirmations_and_remove_caller(&hash, &mut set, &caller);
                set.push(caller);

                let new_status = {
                    if set.len() >= self.requirement as usize {
                        ConfirmationStatus::FullyConfirmed(self.owners_set_hash)
                    } else {
                        ConfirmationStatus::PartialConfirmed(self.owners_set_hash, set)
                    }
                };
                self.transaction_status.insert(trans_id, &new_status);

                self.env().emit_event(Confirmation {
                    transaction_id: trans_id,
                    from: caller,
                    status: new_status.clone(),
                });
                return new_status
            }
            status
        }

        /// Revoke the senders confirmation.
        ///
        /// This can be called by any owner.
        ///
        /// # Panics
        ///
        /// If `trans_id` is no valid transaction id.
        #[ink(message)]
        pub fn revoke_confirmation(&mut self, trans_to_conf: TransactionToConfirm) {
            self.ensure_caller_is_owner();
            let trans_id = trans_to_conf.to_transaction_id();

            // The user can revoke only transaction that is in partial confirmed state
            if let Some(ConfirmationStatus::PartialConfirmed(hash, mut set)) =
                self.transaction_status.get(&trans_id)
            {
                let caller = self.env().caller();
                self.actualize_confirmations_and_remove_caller(&hash, &mut set, &caller);

                let new_status =
                    ConfirmationStatus::PartialConfirmed(self.owners_set_hash, set);
                self.transaction_status.insert(trans_id, &new_status);

                self.env().emit_event(Revokation {
                    transaction_id: trans_id,
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
                transaction_id: trans_id,
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
                transaction_id: trans_id,
                result: result.clone().map(Some),
            });
            result
        }

        /// Get the index of `owner` in `self.owners`.
        /// Panics if `owner` is not found in `self.owners`.
        fn owner_index(&self, owner: &AccountId) -> u32 {
            self.owners.iter().position(|x| x == owner).expect(
                "This is only called after it was already verified that the id is
                 actually an owner.",
            ) as u32
        }

        /// Remove the transaction identified by `trans_id` from `self.transactions`.
        fn take_transaction(&mut self, trans_id: TransactionId) -> Option<Transaction> {
            let transaction = self.transactions.get(&trans_id);
            if transaction.is_some() {
                self.transactions.remove(&trans_id);
            }
            transaction
        }

        /// Panic if transaction `trans_id` is not confirmed by at least
        /// `self.requirement` owners.
        fn ensure_confirmed(&self, trans_id: TransactionId) {
            assert!(match self.transaction_status.get(&trans_id) {
                Some(ConfirmationStatus::FullyConfirmed(_)) => true,
                _ => false,
            });
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
            assert!(self.is_owner(owner));
        }

        /// Panic if `owner` is an owner.
        fn ensure_no_owner(&self, owner: &AccountId) {
            assert!(!self.is_owner(owner));
        }

        fn is_owner(&self, owner: &AccountId) -> bool {
            self.is_owner.get(&owner).is_some()
        }

        fn actualize_confirmations_and_remove_caller(
            &self,
            owners_set_hash: &OwnersSetHash,
            owners: &mut Vec<AccountId>,
            caller: &AccountId,
        ) {
            // It means that set of owners changed and we need to actualize confirmations
            if owners_set_hash != &self.owners_set_hash {
                for i in (0..owners.len()).rev() {
                    if !self.is_owner(&owners[i]) {
                        owners.swap_remove(i);
                    }
                }
            }

            for i in 0..owners.len() {
                if &owners[i] == caller {
                    owners.swap_remove(i);
                    break
                }
            }
        }
    }

    /// Panic if the number of `owners` under a `requirement` violates our
    /// requirement invariant.
    fn ensure_requirement_is_valid(owners: u32, requirement: u32) {
        assert!(0 < requirement && requirement <= owners && owners <= MAX_OWNERS);
    }

    /// Calculate the hash of owners set
    fn owners_set_hash(owners: &Vec<AccountId>) -> OwnersSetHash {
        let mut output = <Blake2x256 as HashOutput>::Type::default();
        ink_env::hash_encoded::<Blake2x256, _>(&owners, &mut output);
        output.into()
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
            assert_eq!(contract.transactions.len(), 1);
            assert_eq!(test::recorded_events().count(), 2);
            let transaction = contract.transactions.get(0).unwrap();
            assert_eq!(*transaction, Transaction::change_requirement(1));
            contract.confirmations.get(&(0, accounts.alice)).unwrap();
            assert_eq!(contract.confirmations.len(), 1);
            assert_eq!(*contract.confirmation_count.get(&0).unwrap(), 1);
            contract
        }

        #[ink::test]
        fn construction_works() {
            let accounts = default_accounts();
            let owners = ink_prelude::vec![accounts.alice, accounts.bob, accounts.eve];
            let contract = build_contract();

            assert_eq!(contract.owners.len(), 3);
            assert_eq!(*contract.requirement, 2);
            assert!(contract.owners.iter().eq(owners.iter()));
            assert!(contract.is_owner.get(&accounts.alice).is_some());
            assert!(contract.is_owner.get(&accounts.bob).is_some());
            assert!(contract.is_owner.get(&accounts.eve).is_some());
            assert!(contract.is_owner.get(&accounts.charlie).is_none());
            assert!(contract.is_owner.get(&accounts.django).is_none());
            assert!(contract.is_owner.get(&accounts.frank).is_none());
            assert_eq!(contract.confirmations.len(), 0);
            assert_eq!(contract.confirmation_count.len(), 0);
            assert_eq!(contract.transactions.len(), 0);
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
            assert_eq!(*contract.requirement, 2);
            set_from_wallet();
            contract.change_requirement(3);
            assert_eq!(*contract.requirement, 3);
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
            assert_eq!(contract.transactions.len(), 0);
            assert_eq!(test::recorded_events().count(), 3);
        }

        #[ink::test]
        fn cancel_transaction_nonexisting() {
            let mut contract = submit_transaction();
            set_from_wallet();
            contract.cancel_transaction(1);
            assert_eq!(contract.transactions.len(), 1);
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
            assert_eq!(contract.confirmations.len(), 2);
            assert_eq!(*contract.confirmation_count.get(&0).unwrap(), 2);
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
            assert_eq!(contract.confirmations.len(), 3);
            assert_eq!(*contract.confirmation_count.get(&0).unwrap(), 3);
            // Revoke from Eve
            contract.revoke_confirmation(0);
            assert_eq!(*contract.confirmation_count.get(&0).unwrap(), 2);
            // Revoke from Bob
            set_sender(accounts.bob);
            contract.revoke_confirmation(0);
            assert_eq!(*contract.confirmation_count.get(&0).unwrap(), 1);
        }

        #[ink::test]
        fn confirm_transaction_already_confirmed() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_sender(accounts.alice);
            contract.confirm_transaction(0);
            assert_eq!(test::recorded_events().count(), 2);
            contract.confirmations.get(&(0, accounts.alice)).unwrap();
            assert_eq!(contract.confirmations.len(), 1);
            assert_eq!(*contract.confirmation_count.get(&0).unwrap(), 1);
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
            assert_eq!(contract.confirmations.len(), 0);
            assert_eq!(*contract.confirmation_count.get(&0).unwrap(), 0);
        }

        #[ink::test]
        fn revoke_transaction_no_confirmer() {
            let mut contract = submit_transaction();
            let accounts = default_accounts();
            set_sender(accounts.bob);
            contract.revoke_confirmation(0);
            assert_eq!(test::recorded_events().count(), 2);
            assert!(contract.confirmations.get(&(0, accounts.alice)).is_some());
            assert_eq!(contract.confirmations.len(), 1);
            assert_eq!(*contract.confirmation_count.get(&0).unwrap(), 1);
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
