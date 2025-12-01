// Copyright (C) Use Ink (UK) Ltd.
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

//! Unified conversion traits for account identifiers and addresses.
//!
//! This module provides two core traits for converting between different
//! account representations used in ink! e2e tests:
//!
//! - [`IntoAccountId`]: Convert to any `AccountId` type (32-byte identifiers)
//! - [`IntoAddress`]: Convert to an Ethereum-style `Address` (H160, 20 bytes)
//!
//! All conversions ultimately flow through `[u8; 32]` as the canonical
//! intermediate representation.

use crate::sr25519::Keypair;
use ink_primitives::{
    Address,
    types::AccountIdMapper,
};
use sp_core::crypto::AccountId32;

// ═══════════════════════════════════════════════════════════════════════════
// IntoAccountId - Generic conversion to any AccountId type
// ═══════════════════════════════════════════════════════════════════════════

/// Trait for types that can be converted into an `AccountId`.
///
/// This trait enables flexible API signatures that accept multiple account
/// types without requiring manual conversions from callers.
///
/// # Example
///
/// ```ignore
/// fn do_something(account: impl IntoAccountId<AccountId32>) {
///     let account_id: AccountId32 = account.into_account_id();
///     // ...
/// }
///
/// // All of these work:
/// do_something(&alice());           // Keypair
/// do_something(&account_id);        // ink_primitives::AccountId
/// do_something(&account_id_32);     // AccountId32
/// ```
pub trait IntoAccountId<TargetAccountId> {
    fn into_account_id(self) -> TargetAccountId;
}

// Identity conversion for AccountId32
impl IntoAccountId<AccountId32> for AccountId32 {
    fn into_account_id(self) -> AccountId32 {
        self
    }
}

// Borrow conversion for AccountId32
impl IntoAccountId<AccountId32> for &AccountId32 {
    fn into_account_id(self) -> AccountId32 {
        self.clone()
    }
}

// Generic conversion from ink_primitives::AccountId
impl<AccountId> IntoAccountId<AccountId> for &ink_primitives::AccountId
where
    AccountId: From<[u8; 32]>,
{
    fn into_account_id(self) -> AccountId {
        AccountId::from(*AsRef::<[u8; 32]>::as_ref(self))
    }
}

// Generic conversion from Keypair (sr25519)
impl<AccountId> IntoAccountId<AccountId> for &Keypair
where
    AccountId: From<[u8; 32]>,
{
    fn into_account_id(self) -> AccountId {
        AccountId::from(self.public_key().0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// IntoAddress - Convert to Ethereum-style Address (H160)
// ═══════════════════════════════════════════════════════════════════════════

/// Extension trait for converting various types to Address (H160).
///
/// The conversion uses [`AccountIdMapper::to_address`] which handles both:
/// - Ethereum-derived accounts (last 12 bytes are `0xEE`): extracts first 20 bytes
/// - Sr25519-derived accounts: keccak256 hash, then takes last 20 bytes
///
/// # Example
///
/// ```ignore
/// use ink_e2e::{IntoAddress, alice};
///
/// let address = alice().address();
/// ```
pub trait IntoAddress {
    /// Convert to an Address (H160).
    fn address(&self) -> Address;
}

impl IntoAddress for Keypair {
    fn address(&self) -> Address {
        AccountIdMapper::to_address(&self.public_key().0)
    }
}

impl IntoAddress for ink_primitives::AccountId {
    fn address(&self) -> Address {
        let bytes = *AsRef::<[u8; 32]>::as_ref(self);
        AccountIdMapper::to_address(&bytes)
    }
}

impl IntoAddress for AccountId32 {
    fn address(&self) -> Address {
        AccountIdMapper::to_address(self.as_ref())
    }
}
