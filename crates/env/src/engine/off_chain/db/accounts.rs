// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{
    super::{
        OffChainError,
        TypedEncodedError,
    },
    OffAccountId,
    OffBalance,
};
use crate::{
    Environment,
    Error,
};
use core::cell::Cell;
use derive_more::From;
use ink_prelude::collections::BTreeMap;
use ink_primitives::Key;

/// Errors encountered upon interacting with the accounts database.
#[derive(Debug, From, PartialEq, Eq)]
pub enum AccountError {
    TypedEncoded(TypedEncodedError),
    #[from(ignore)]
    UnexpectedUserAccount,
    #[from(ignore)]
    NoAccountForId(OffAccountId),
}

impl From<AccountError> for Error {
    fn from(account_error: AccountError) -> Self {
        Error::OffChain(OffChainError::Account(account_error))
    }
}

impl AccountError {
    /// Creates a new error to indicate a missing account.
    pub fn no_account_for_id<T>(account_id: &T::AccountId) -> Self
    where
        T: Environment,
    {
        Self::NoAccountForId(OffAccountId::new(account_id))
    }
}

impl From<scale::Error> for AccountError {
    fn from(err: scale::Error) -> Self {
        AccountError::TypedEncoded(err.into())
    }
}

/// Result type encountered while operating on accounts.
pub type Result<T> = core::result::Result<T, AccountError>;

/// The database that stores all accounts.
pub struct AccountsDb {
    /// The mapping from account ID to an actual account.
    accounts: BTreeMap<OffAccountId, Account>,
}

impl AccountsDb {
    /// Creates a new empty accounts database.
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }

    /// Resets the account DB to uninitialized state.
    pub fn reset(&mut self) {
        self.accounts.clear()
    }

    /// Returns the account at the given account ID or creates it.
    pub fn get_or_create_account<T>(&mut self, at: &T::AccountId) -> &mut Account
    where
        T: Environment,
    {
        // Note: We cannot do a normal match for `Some(account)` here since
        //       the borrow-checker somehow cannot make sense of it according
        //       to its lifetime analysis. Consider this to be a hack until
        //       the borrow-checker eventually let's us do this.
        if self.get_account::<T>(&at).is_some() {
            self.get_account_mut::<T>(at)
                .expect("just checked that account exists")
        } else {
            self.add_user_account::<T>(at.clone(), 0u32.into());
            self.get_account_mut::<T>(at)
                .expect("just added the account so it must exist")
        }
    }

    /// Returns the account for the given account ID if any.
    pub fn get_account<T>(&self, at: &T::AccountId) -> Option<&Account>
    where
        T: Environment,
    {
        self.accounts.get(&OffAccountId::new(at))
    }

    /// Returns the account for the given account ID if any.
    pub fn get_account_mut<T>(&mut self, at: &T::AccountId) -> Option<&mut Account>
    where
        T: Environment,
    {
        self.accounts.get_mut(&OffAccountId::new(at))
    }

    /// Returns the account for the given off-account ID if any.
    pub fn get_account_off<'a>(&'a self, at: &OffAccountId) -> Option<&'a Account> {
        self.accounts.get(at)
    }

    /// Returns the account for the given off-account ID if any.
    pub fn get_account_off_mut(&mut self, at: &OffAccountId) -> Option<&mut Account> {
        self.accounts.get_mut(at)
    }

    /// Adds the given user account with the initial balance.
    pub fn add_user_account<T>(
        &mut self,
        account_id: T::AccountId,
        initial_balance: T::Balance,
    ) where
        T: Environment,
    {
        self.accounts.insert(
            OffAccountId::new(&account_id),
            Account {
                balance: OffBalance::new(&initial_balance),
                kind: AccountKind::User,
            },
        );
    }

    /// Creates a new contract account.
    pub fn add_contract_account<T>(
        &mut self,
        account_id: T::AccountId,
        initial_balance: T::Balance,
        rent_allowance: T::Balance,
    ) where
        T: Environment,
    {
        self.accounts.insert(
            OffAccountId::new(&account_id),
            Account {
                balance: OffBalance::new(&initial_balance),
                kind: AccountKind::Contract(ContractAccount::new::<T>(rent_allowance)),
            },
        );
    }

    /// Removes an account.
    pub fn remove_account<T>(&mut self, account_id: T::AccountId)
    where
        T: Environment,
    {
        self.accounts.remove(&OffAccountId::new(&account_id));
    }
}

/// An account within the chain.
pub struct Account {
    /// The balance of the account.
    balance: OffBalance,
    /// The kind of the account.
    kind: AccountKind,
}

impl Account {
    /// Returns the balance of the account.
    pub fn balance<T>(&self) -> Result<T::Balance>
    where
        T: Environment,
    {
        self.balance.decode().map_err(Into::into)
    }

    /// Sets the balance of the account.
    pub fn set_balance<T>(&mut self, new_balance: T::Balance) -> Result<()>
    where
        T: Environment,
    {
        self.balance.assign(&new_balance).map_err(Into::into)
    }

    /// Returns the contract account or an error if it is a user account.
    fn contract_or_err(&self) -> Result<&ContractAccount> {
        match &self.kind {
            AccountKind::User => {
                Err(AccountError::UnexpectedUserAccount).map_err(Into::into)
            }
            AccountKind::Contract(contract_account) => Ok(contract_account),
        }
    }

    /// Returns the contract account or an error if it is a user account.
    fn contract_or_err_mut(&mut self) -> Result<&mut ContractAccount> {
        match &mut self.kind {
            AccountKind::User => {
                Err(AccountError::UnexpectedUserAccount).map_err(Into::into)
            }
            AccountKind::Contract(contract_account) => Ok(contract_account),
        }
    }

    /// Returns the rent allowance of the contract account or an error.
    pub fn rent_allowance<T>(&self) -> Result<T::Balance>
    where
        T: Environment,
    {
        self.contract_or_err()
            .and_then(|contract| contract.rent_allowance.decode().map_err(Into::into))
    }

    /// Sets the rent allowance for the contract account or returns an error.
    pub fn set_rent_allowance<T>(&mut self, new_rent_allowance: T::Balance) -> Result<()>
    where
        T: Environment,
    {
        self.contract_or_err_mut().and_then(|contract| {
            contract
                .rent_allowance
                .assign(&new_rent_allowance)
                .map_err(Into::into)
        })
    }

    /// Sets the contract storage of key to the new value.
    pub fn set_storage<T>(&mut self, at: Key, new_value: &T) -> Result<()>
    where
        T: scale::Encode,
    {
        self.contract_or_err_mut()
            .map(|contract| contract.storage.set_storage::<T>(at, new_value))
    }

    /// Clears the contract storage at key.
    pub fn clear_storage(&mut self, at: Key) -> Result<()> {
        self.contract_or_err_mut()
            .map(|contract| contract.storage.clear_storage(at))
    }

    /// Returns the value stored in the contract storage at the given key.
    pub fn get_storage<T>(&self, at: Key) -> Result<Option<T>>
    where
        T: scale::Decode,
    {
        self.contract_or_err()
            .and_then(|contract| contract.storage.get_storage::<T>(at))
    }

    /// Returns the total number of reads and write from and to the contract's storage.
    pub fn get_storage_rw(&self) -> Result<(usize, usize)> {
        self.contract_or_err().map(|contract| contract.get_rw())
    }

    /// Returns the amount of used storage entries.
    pub fn count_used_storage_cells(&self) -> Result<usize> {
        self.contract_or_err()
            .map(|contract| contract.count_used_storage_cells())
    }
}

/// The kind of the account.
///
/// Can be either a user account or a (more complicated) contract account.
pub enum AccountKind {
    User,
    Contract(ContractAccount),
}

/// Extraneous fields for contract accounts.
pub struct ContractAccount {
    /// The contract's rent allowance.
    rent_allowance: OffBalance,
    /// The contract storage.
    pub storage: ContractStorage,
}

impl ContractAccount {
    /// Creates a new contract account with the given initial rent allowance.
    pub fn new<T>(rent_allowance: T::Balance) -> Self
    where
        T: Environment,
    {
        Self {
            rent_allowance: OffBalance::new(&rent_allowance),
            storage: ContractStorage::new(),
        }
    }

    /// Returns the number of reads and writes from and to the contract storage.
    pub fn get_rw(&self) -> (usize, usize) {
        self.storage.get_rw()
    }

    /// Returns the number of used storage entries.
    pub fn count_used_storage_cells(&self) -> usize {
        self.storage.count_used_storage_cells()
    }
}

/// The storage of a contract instance.
pub struct ContractStorage {
    /// The entries within the contract storage.
    entries: BTreeMap<Key, Vec<u8>>,
    /// The total number of reads to the storage.
    count_reads: Cell<usize>,
    /// The total number of writes to the storage.
    count_writes: usize,
}

impl ContractStorage {
    /// Creates a new empty contract storage.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            count_reads: Cell::new(0),
            count_writes: 0,
        }
    }

    /// Returns the number of reads and writes from and to the contract storage.
    pub fn get_rw(&self) -> (usize, usize) {
        (self.count_reads.get(), self.count_writes)
    }

    /// Returns the decoded storage at the key if any.
    pub fn get_storage<T>(&self, at: Key) -> Result<Option<T>>
    where
        T: scale::Decode,
    {
        self.count_reads.set(self.count_reads.get() + 1);
        self.entries
            .get(&at)
            .map(|encoded| T::decode(&mut &encoded[..]))
            .transpose()
            .map_err(Into::into)
    }

    /// Writes the encoded value into the contract storage at the given key.
    pub fn set_storage<T>(&mut self, at: Key, new_value: &T)
    where
        T: scale::Encode,
    {
        self.count_writes += 1;
        self.entries.insert(at, new_value.encode());
    }

    /// Removes the value from storage entries at the given key.
    pub fn clear_storage(&mut self, at: Key) {
        self.count_writes += 1;
        self.entries.remove(&at);
    }

    /// Returns the number of used storage entries.
    pub fn count_used_storage_cells(&self) -> usize {
        self.entries.len()
    }
}
