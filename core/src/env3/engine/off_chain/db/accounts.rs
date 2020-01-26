// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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
    OffHash,
};
use crate::{
    env3::{
        EnvError,
        EnvTypes,
    },
    storage::Key,
};
use derive_more::From;
use ink_prelude::collections::BTreeMap;

/// Errors encountered upon interacting with the accounts database.
#[derive(Debug, From)]
pub enum AccountError {
    TypedEncoded(TypedEncodedError),
    #[from(ignore)]
    UnexpectedUserAccount,
    #[from(ignore)]
    NoAccountForId(OffAccountId),
}

impl From<AccountError> for EnvError {
    fn from(account_error: AccountError) -> Self {
        EnvError::OffChain(OffChainError::Account(account_error))
    }
}

impl AccountError {
    /// Creates a new error to indicate a missing account.
    pub fn no_account_for_id<T>(account_id: &T::AccountId) -> Self
    where
        T: EnvTypes,
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

    /// Returns the account for the given account ID if any.
    pub fn get_account<T>(&self, at: &T::AccountId) -> Option<&Account>
    where
        T: EnvTypes,
    {
        self.accounts.get(&OffAccountId::new(at))
    }

    /// Returns the account for the given account ID if any.
    pub fn get_account_mut<T>(&mut self, at: &T::AccountId) -> Option<&mut Account>
    where
        T: EnvTypes,
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

    /// Creates a new user account.
    pub fn new_user_account<T>(&mut self, initial_balance: T::Balance) -> T::AccountId
    where
        T: EnvTypes,
    {
        todo!()
    }

    /// Creates a new contract account.
    pub fn new_contract_account<T>(&mut self) -> T::AccountId
    where
        T: EnvTypes,
    {
        todo!()
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
        T: EnvTypes,
    {
        self.balance.decode().map_err(Into::into)
    }

    /// Sets the balance of the account.
    pub fn set_balance<T>(&mut self, new_balance: T::Balance) -> Result<()>
    where
        T: EnvTypes,
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
        T: EnvTypes,
    {
        self.contract_or_err()
            .and_then(|contract| contract.rent_allowance.decode().map_err(Into::into))
    }

    /// Sets the rent allowance for the contract account or returns an error.
    pub fn set_rent_allowance<T>(&mut self, new_rent_allowance: T::Balance) -> Result<()>
    where
        T: EnvTypes,
    {
        self.contract_or_err_mut().and_then(|contract| {
            contract
                .rent_allowance
                .assign(&new_rent_allowance)
                .map_err(Into::into)
        })
    }

    /// Returns the code hash of the contract account of an error.
    pub fn code_hash<T>(&self) -> Result<T::Hash>
    where
        T: EnvTypes,
    {
        self.contract_or_err()
            .and_then(|contract| contract.code_hash.decode().map_err(Into::into))
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

    /// Clears the contract storage at key.
    pub fn get_storage<T>(&self, at: Key) -> Result<Option<T>>
    where
        T: scale::Decode,
    {
        self.contract_or_err()
            .and_then(|contract| contract.storage.get_storage::<T>(at))
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
    /// The contract's code hash.
    code_hash: OffHash,
    /// The contract storage.
    pub storage: ContractStorage,
}

/// The storage of a contract instance.
pub struct ContractStorage {
    /// The entries within the contract storage.
    entries: BTreeMap<Key, Vec<u8>>,
}

impl ContractStorage {
    /// Returns the decoded storage at the key if any.
    pub fn get_storage<T>(&self, at: Key) -> Result<Option<T>>
    where
        T: scale::Decode,
    {
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
        self.entries.insert(at, new_value.encode());
    }

    /// Removes the value from storage entries at the given key.
    pub fn clear_storage(&mut self, at: Key) {
        self.entries.remove(&at);
    }
}
