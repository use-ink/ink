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

//! Default definitions for contract environment types without a
//! dependency on a substrate runtime.
//!
//! # Note
//!
//! These types can only be used safely when they are compatible with
//! the corresponding type defined in the target contract runtime

#![cfg_attr(not(any(test, feature = "test-env")), no_std)]

use ink_core::env::EnvTypes;
use parity_codec::{
    Decode,
    Encode,
};

/// The default contract environment types.
#[allow(unused)]
pub enum DefaultEnvTypes {}

impl EnvTypes for DefaultEnvTypes {
    type AccountId = AccountId;
    type Balance = Balance;
    type Hash = Hash;
    type Moment = Moment;
}

/// The default address type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct AccountId([u8; 32]);

/// The default balance type.
pub type Balance = u64;

/// The default hash type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct Hash([u8; 32]);

/// The default moment type.
pub type Moment = u64;
