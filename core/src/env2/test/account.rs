// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    env2::{
        call::{
            CallData,
            Selector,
        },
        test::{
            Storage,
            TypedEncoded,
            types::*,
        },
    },
    memory::collections::btree_map::BTreeMap,
};

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

    /// Returns the account associated with the given ID and otherwise returns `None`.
    pub fn get(&self, account_id: &AccountId) -> Option<&Account> {
        self.accounts.get(account_id)
    }

    /// Returns the account associated with the given ID and otherwise returns `None`.
    pub fn get_mut(&mut self, account_id: &AccountId) -> Option<&mut Account> {
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
        return None
    }

    /// Returns the user account if `self` is a user account and otherwise return `None`.
    pub fn user_mut(&mut self) -> Option<&mut UserAccount> {
        if let AccountKind::User(user_account) = &mut self.kind {
            return Some(user_account)
        }
        return None
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
        return None
    }

    /// Returns the user account if `self` is a user account and otherwise return `None`.
    pub fn contract_mut(&mut self) -> Option<&mut ContractAccount> {
        if let AccountKind::Contract(contract_account) = &mut self.kind {
            return Some(contract_account)
        }
        return None
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
