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

//! Emulated environmental types of the test environment.
//!
//! Due to technical constraints it is not possible to define the test
//! environment instance thread-locally and generically over the actual
//! environmental types.
//!
//! For that reason we store the type information of all the environmental
//! types at runtime to at least have some kind of type safety upon access.
//! This is done via the `TypedEncoded` abstraction that stores the
//! SCALE encoded bytes and also has a runtime type information marker
//! assigned upon initialization to check whether accesses to it are
//! type safe.

use crate::env2::test::TypedEncoded;

/// Type markers used in conjunction with `TypedEncoded`.
#[rustfmt::skip]
mod type_marker {
    /// Type marker representing an environmental `AccountId`.
    #[derive(Debug)] pub enum AccountId {}
    /// Type marker representing an environmental `Balance`.
    #[derive(Debug)] pub enum Balance {}
    /// Type marker representing an environmental `Hash`.
    #[derive(Debug)] pub enum Hash {}
    /// Type marker representing an environmental `Moment`.
    #[derive(Debug)] pub enum Moment {}
    /// Type marker representing an environmental `BlockNumber`.
    #[derive(Debug)] pub enum BlockNumber {}
    /// Type marker representing an environmental `Call`.
    #[derive(Debug)] pub enum Call {}
}

/// Environmental account ID type.
pub type AccountId = TypedEncoded<type_marker::AccountId>;
/// Environmental balance type.
pub type Balance = TypedEncoded<type_marker::Balance>;
/// Environmental hash type.
pub type Hash = TypedEncoded<type_marker::Hash>;
/// Environmental moment (block time) type.
pub type Moment = TypedEncoded<type_marker::Moment>;
/// Environmental block number type.
pub type BlockNumber = TypedEncoded<type_marker::BlockNumber>;
/// Environmental call (runtime dispatch) type.
pub type Call = TypedEncoded<type_marker::Call>;
