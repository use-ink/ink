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
pub struct Accounts {
    /// All on-chain registered accounts.
    accounts: BTreeMap<AccountId, Account>,
}

impl Default for Accounts {
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
    balance: Balance,
    /// The kind of the account and associated data.
    kind: AccountKind,
}

impl Account {
    /// Returns `true` if `self` is a user account.
    pub fn is_user(&self) -> bool {
        if let AccountKind::User(_) = self.kind {
            return true
        }
        false
    }

    /// Returns `true` if `self` is a contract account.
    pub fn is_contract(&self) -> bool {
        if let AccountKind::Contract(_) = self.kind {
            return true
        }
        false
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
    display_name: String,
}

/// Specific state of contract accounts.
#[derive(Debug, Clone)]
pub struct ContractAccount {
    /// The associated code hash.
    code_hash: Hash,
    /// The contract's unique storage.
    storage: Storage,
    /// The minimum balance allowed for the contract.
    minimum_balance: Balance,
}
