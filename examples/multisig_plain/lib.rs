//! # Plain Multisig Wallet
//!
//! This implements a plain multi owner wallet.
//!
//! ## Overview
//!
//! Each instantiation of this contract has a set of `owners` and a `requirement` of
//! how many of them need to agree on a `Transaction` for it to be able to be executed.
//! Every owner can submit a transaction and when enough of the other owners confirm
//! it will be able to be executed. The following invariant is enforced by the contract:
//!
//! ```0 < requirement && requirement <= owners && owners <= MAX_OWNERS```
//!
//! ## Error Handling
//!
//! With the exeception of `execute_transaction` no error conditions are signalled
//! through return types. Any error or invariant violation triggers a panic and therefore
//! rolls back the transaction.
//!
//! ## Interface
//!
//! The interface is modelled after the popular gnosis multisig wallet. However, there
//! are subtle variations from the interface. For example the `confirm_transaction`
//! will never trigger the execution of a `Transaction` even if the treshold is reached.
//! A call of `execute_transaction` is always required. This can be called by anyone.
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

use ink_lang as ink;

#[ink::contract(version = "0.1.0", env = MyEnv)]
mod multisig_plain {
    use ink_core::{
        env,
        storage,
    };
    use ink_prelude::vec::Vec;
    use scale::Output;

    /// When using custom types in your runtime. Here is the place to declare them.
    type MyEnv = env::DefaultEnvTypes;
    /// Tune this to your liking but be that many owners will not perform well.
    const MAX_OWNERS: u32 = 50;

    type TransactionId = u32;
    const WRONG_TRANSACTION_ID: &str =
        "The user specified an invalid transaction id. Abort.";

    /// A wrapper that allows us to pass untyped parameters as blob to a `CallBuilder`
    struct CallInput<'a>(&'a [u8]);

    impl<'a> scale::Encode for CallInput<'a> {
        fn encode_to<T: Output>(&self, dest: &mut T) {
            dest.write(self.0);
        }
    }

    /// A Transaction is what every `owner` can submit for confirmation by other owners.
    /// If enough owners agree it will be executed by the contract.
    #[derive(scale::Encode, scale::Decode, storage::Flush)]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub struct Transaction {
        /// The AccountId of the contract that is called in this transaction.
        callee: AccountId,
        /// The raw selector which is the function name of the `callee`that is called.
        selector: [u8; 4],
        /// The raw parameters that are passed to the called function.
        input: Vec<u8>,
        /// The amount of chain balance that is transferred to the callee.
        transferred_value: Balance,
        /// Gas limit for the transation.
        gas_limit: u64,
    }

    #[ink(storage)]
    struct MultisigPlain {
        /// Every entry in this map represents the confirmation of an owner for a
        /// transaction. This is effecively a set rather than a map.
        confirmations: storage::HashMap<(TransactionId, AccountId), ()>,
        /// The amount of confirmations for every transaction. This is a redundant
        /// information this kept in order to prevent iterating through the
        /// confirmation set to check if a transaction is confirmed.
        confirmation_count: storage::HashMap<TransactionId, u32>,
        /// Just the list of transactions. It is a stash as stable ids are necessary
        /// for referencing them in confirmation calls.
        transactions: storage::Stash<Transaction>,
        /// The list is a vector because iterating over it is necessary when cleaning
        /// up the confirmation set.
        owners: storage::Vec<AccountId>,
        /// Redundent information to speed up the check whether a caller is an owner.
        is_owner: storage::HashMap<AccountId, ()>,
        /// Minimum number of owners that have to confirm a transaction to be executed.
        requirement: storage::Value<u32>,
    }

    impl MultisigPlain {
        /// The only constructor the the contract. A list of owners must be supplied
        /// and a number of how many of them must confirm a transaction. Duplicate
        /// owners are silently dropped.
        #[ink(constructor)]
        fn new(&mut self, owners: Vec<AccountId>, requirement: u32) {
            for owner in &owners {
                self.is_owner.insert(*owner, ());
                self.owners.push(*owner);
            }
            ensure_requirement_is_valid(self.owners.len(), requirement);
            assert!(self.is_owner.len() == self.owners.len());
            self.requirement.set(requirement);
        }

        /// Add a new owner to the contract. Fails is the owner already exists.
        /// Only callable by the wallet itself.
        #[ink(message)]
        fn add_owner(&mut self, new_owner: AccountId) {
            self.ensure_from_wallet();
            self.ensure_no_owner(&new_owner);
            ensure_requirement_is_valid(self.owners.len() + 1, *self.requirement);
            self.is_owner.insert(new_owner, ());
            self.owners.push(new_owner);
        }

        /// Remove an owner from the contract.
        /// Only callable by the wallet itself. If by doing this the amount of owners
        /// would be smaller than the requirement it is adjusted to be exactly the
        /// number of owners.
        #[ink(message)]
        fn remove_owner(&mut self, owner: AccountId) {
            self.ensure_from_wallet();
            self.ensure_owner(&owner);
            let len = self.owners.len() - 1;
            let requirement = u32::min(len, *self.requirement.get());
            ensure_requirement_is_valid(len, requirement);
            self.owners.swap_remove(self.owner_index(&owner));
            self.is_owner.remove(&owner);
            self.requirement.set(requirement);
            self.clean_owner_confirmations(&owner);
        }

        /// Replace an owner from the contract with a new one.
        /// Only callable by the wallet itself.
        #[ink(message)]
        fn replace_owner(&mut self, old_owner: AccountId, new_owner: AccountId) {
            self.ensure_from_wallet();
            self.ensure_owner(&old_owner);
            self.ensure_no_owner(&new_owner);
            self.owners
                .replace(self.owner_index(&old_owner), || new_owner);
            self.is_owner.remove(&old_owner);
            self.is_owner.insert(new_owner, ());
            self.clean_owner_confirmations(&old_owner);
        }

        /// Change the requirement to a new value.
        /// Only callable by the wallet itself.
        #[ink(message)]
        fn change_requirement(&mut self, new_requirement: u32) {
            self.ensure_from_wallet();
            ensure_requirement_is_valid(self.owners.len(), new_requirement);
            self.requirement.set(new_requirement);
        }

        /// Add a new transaction candiate to the contract.
        /// This also confirms the transaction for the caller.
        /// This can be called by any owner.
        #[ink(message)]
        fn submit_transaction(&mut self, transaction: Transaction) {
            self.ensure_caller_is_owner();
            let trans_id = self.transactions.put(transaction);
            self.confirmation_count.insert(trans_id, 0);
            self.confirm_by_caller(self.env().caller(), trans_id);
        }

        /// Remove a transaction from the contract.
        /// Only callable by the wallet itself.
        #[ink(message)]
        fn cancel_transaction(&mut self, trans_id: TransactionId) {
            self.ensure_from_wallet();
            self.take_transaction(trans_id);
        }

        /// Confirm a transaction for the sender that was submitted by any owner.
        /// This can be called by any owner.
        #[ink(message)]
        fn confirm_transaction(&mut self, trans_id: TransactionId) {
            self.ensure_caller_is_owner();
            self.ensure_transaction_exists(trans_id);
            self.confirm_by_caller(self.env().caller(), trans_id);
        }

        /// Revoke the senders confirmation.
        /// This can be called by any owner.
        #[ink(message)]
        fn revoke_confirmation(&mut self, trans_id: TransactionId) {
            self.ensure_caller_is_owner();
            if self
                .confirmations
                .remove(&(trans_id, self.env().caller()))
                .is_some()
            {
                self.confirmation_count
                    .mutate_with(&trans_id, |count| *count -= 1);
            }
        }

        /// Execute a already confirmed execution. Its return type indicates whether
        /// the called transaction was succesful.
        /// This can be called by anyone.
        #[ink(message)]
        fn execute_transaction(&mut self, trans_id: TransactionId) -> Result<(), ()> {
            self.ensure_confirmed(trans_id);
            let t = self.take_transaction(trans_id).expect(WRONG_TRANSACTION_ID);
            env::call::CallParams::<MyEnv, ()>::invoke(t.callee, t.selector.into())
                .gas_limit(t.gas_limit)
                .transferred_value(t.transferred_value)
                .push_arg(&CallInput(&t.input))
                .fire()
                .map(|_| ())
                .map_err(|_| ())
        }

        fn confirm_by_caller(
            &mut self,
            confirmer: AccountId,
            transaction: TransactionId,
        ) {
            if self
                .confirmations
                .insert((transaction, confirmer), ())
                .is_none()
            {
                self.confirmation_count
                    .mutate_with(&transaction, |count| *count += 1);
            }
        }

        fn owner_index(&self, owner: &AccountId) -> u32 {
            self.owners.iter().position(|x| *x == *owner).expect(
                "This is only called after it was already verified that the id is
                actually an owner.",
            ) as u32
        }

        fn take_transaction(&mut self, trans_id: TransactionId) -> Option<Transaction> {
            let transaction = self.transactions.take(trans_id);
            if transaction.is_some() {
                self.clean_transaction_confirmations(trans_id);
            }
            transaction
        }

        fn clean_owner_confirmations(&mut self, owner: &AccountId) {
            for (trans_id, _) in self.transactions.iter() {
                if self.confirmations.remove(&(trans_id, *owner)).is_some() {
                    self.confirmation_count
                        .mutate_with(&trans_id, |count| *count -= 1);
                }
            }
        }

        fn clean_transaction_confirmations(&mut self, transaction: TransactionId) {
            for owner in self.owners.iter() {
                self.confirmations.remove(&(transaction, *owner));
            }
            self.confirmation_count.remove(&transaction);
        }

        fn ensure_confirmed(&self, trans_id: TransactionId) {
            assert!(
                self.confirmation_count
                    .get(&trans_id)
                    .expect(WRONG_TRANSACTION_ID)
                    >= self.requirement.get()
            );
        }

        fn ensure_transaction_exists(&self, trans_id: TransactionId) {
            self.transactions.get(trans_id).expect(WRONG_TRANSACTION_ID);
        }

        fn ensure_caller_is_owner(&self) {
            assert!(self.is_owner.contains_key(&self.env().caller()));
        }

        fn ensure_from_wallet(&self) {
            assert!(self.env().caller() == self.env().account_id());
        }

        fn ensure_owner(&self, owner: &AccountId) {
            assert!(self.is_owner.contains_key(owner));
        }

        fn ensure_no_owner(&self, owner: &AccountId) {
            assert!(!self.is_owner.contains_key(owner));
        }
    }

    fn ensure_requirement_is_valid(owners: u32, requirement: u32) {
        assert!(0 < requirement && requirement <= owners && owners <= MAX_OWNERS);
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_core::env::test;
        type Accounts = test::DefaultAccounts<MyEnv>;

        #[test]
        fn construction_works() {
            test::run_test(|accounts: Accounts| {
                MultisigPlain::new(
                    ink_prelude::vec![accounts.alice, accounts.bob, accounts.eve],
                    2,
                );
                Ok(())
            })
            .unwrap();
        }
    }
}
