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

//! The accounts database stores all user- and contract accounts
//! of the emulated block chain.
//!
//! It can also be used to store new accounts and query information
//! about existing ones.
//!
//! The executed smart contract as well as the default caller will
//! always have predefined accounts.

use core::borrow::Borrow;

use crate::env2::test::{
    storage::Storage,
    types::*,
    TypedEncoded,
};
use ink_prelude::collections::btree_map::BTreeMap;

/// The on-chain registered accounts.
///
/// An account can be either a user account or a contract account.
#[derive(Debug, Clone)]
pub struct AccountsDb {
    /// All on-chain registered accounts.
    accounts: BTreeMap<AccountId, Account>,
}

impl AccountsDb {
    /// Returns the number of accounts in the database.
    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    /// Returns `true` if the number of accounts in the database is 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Inserts a new account to the data base.
    pub fn insert(&mut self, account_id: AccountId, account: Account) {
        self.accounts.insert(account_id, account);
    }

    /// Returns the account associated with the given ID and otherwise returns `None`.
    pub fn get<Q: ?Sized>(&self, account_id: &Q) -> Option<&Account>
    where
        AccountId: Borrow<Q>,
        Q: Ord,
    {
        self.accounts.get(account_id)
    }

    /// Returns the account associated with the given ID and otherwise returns `None`.
    pub fn get_mut<Q: ?Sized>(&mut self, account_id: &Q) -> Option<&mut Account>
    where
        AccountId: Borrow<Q>,
        Q: Ord,
    {
        self.accounts.get_mut(account_id)
    }
}

impl Default for AccountsDb {
    fn default() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }
}

/// An on-chain registered account.
#[derive(Debug, Clone)]
pub struct Account {
    /// The current balance of the account.
    pub balance: Balance,
    /// The rent allowance.
    ///
    /// This is not only valid for contract accounts per se.
    pub rent_allowance: Balance,
    /// The kind of the account and associated data.
    pub kind: AccountKind,
}

impl Account {
    /// Returns `true` if `self` is a user account.
    pub fn is_user(&self) -> bool {
        if let AccountKind::User(_) = &self.kind {
            return true
        }
        false
    }

    /// Returns the user account if `self` is a user account and otherwise return `None`.
    pub fn user(&self) -> Option<&UserAccount> {
        if let AccountKind::User(user_account) = &self.kind {
            return Some(user_account)
        }
        None
    }

    /// Returns the user account if `self` is a user account and otherwise return `None`.
    pub fn user_mut(&mut self) -> Option<&mut UserAccount> {
        if let AccountKind::User(user_account) = &mut self.kind {
            return Some(user_account)
        }
        None
    }

    /// Returns `true` if `self` is a contract account.
    pub fn is_contract(&self) -> bool {
        if let AccountKind::Contract(_) = &self.kind {
            return true
        }
        false
    }

    /// Returns the user account if `self` is a user account and otherwise return `None`.
    pub fn contract(&self) -> Option<&ContractAccount> {
        if let AccountKind::Contract(contract_account) = &self.kind {
            return Some(contract_account)
        }
        None
    }

    /// Returns the user account if `self` is a user account and otherwise return `None`.
    pub fn contract_mut(&mut self) -> Option<&mut ContractAccount> {
        if let AccountKind::Contract(contract_account) = &mut self.kind {
            return Some(contract_account)
        }
        None
    }
}

/// The kind of an account.
///
/// An account can be either a user account or a contract account.
#[derive(Debug, Clone)]
pub enum AccountKind {
    /// A user account.
    User(UserAccount),
    /// A contract account.
    Contract(ContractAccount),
}

/// Specific state of user accounts.
#[derive(Debug, Clone)]
pub struct UserAccount {
    /// The users display name.
    pub display_name: String,
}

impl UserAccount {
    /// Creates a new user account.
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            display_name: name.into(),
        }
    }
}

/// Specific state of contract accounts.
#[derive(Debug, Clone)]
pub struct ContractAccount {
    /// The associated code hash.
    pub code_hash: Hash,
    /// The contract's unique storage.
    pub storage: Storage,
    /// The minimum balance allowed for the contract.
    pub minimum_balance: Balance,
}

impl ContractAccount {
    /// Creates a new contract account.
    pub fn new(code_hash: Hash) -> Self {
        Self {
            code_hash,
            storage: Storage::default(),
            minimum_balance: TypedEncoded::from_origin(&0),
        }
    }
}
