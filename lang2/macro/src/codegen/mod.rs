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

//! Generate principles about the code generation of ink!.
//!
//! The user provided module that has the `#[ink::contract]` is called
//! the ink! module.
//!
//! - User provided definitions such as the `#[ink(event)]` definitions
//!   but also non-ink! definitions shall be re-generated at the root of the ink! module.
//!   The only modification on these user provided definitions is the
//!   removal of `#[ink(..)]` attributes. The only exception to
//!   this rule is the `#[ink(storage)] struct` that is going to be generated
//!   within `__ink_private` module because of required structural changes as
//!   well as to restrict access to certain implementation details.
//! - All utility and helper definitions used exclusively by ink!
//!   shall be defined within the `__ink_private` module that itself
//!   is defined as child of the ink! module. Further sub-modules
//!   inside this structure are allowed to further restrict scopes.
//! - Code genenration shall avoid introducing new names and instead rely on
//!   already given names and definitions or use techniques such as using
//!   generic utility structures and `[(); N]` generics where `N` is a unique
//!   hash of some required entity.
//! - Code shall generated with respect to runtime efficiency to not waste gas
//!   upon contract execution. For this the generated code shall try to use
//!   compile-time execution friendly routines that the compiler is known
//!   to handle efficiently.
//! - Generated code shall never conflict with user provided code. This goes
//!   hand-in-hand with avoiding name clashes but further says that types shall
//!   not implement non-ink! traits if not necessary and instead rely optionally
//!   on the user to define them.

mod abi;
mod contract;
mod cross_calling;
mod dispatch;
mod env_types;
mod events;
mod storage;
mod testable;

use crate::ir;
use proc_macro2::TokenStream as TokenStream2;

/// Types implementing this trait are code generators for the ink! language.
pub trait GenerateCode {
    /// Generates ink! contract code.
    fn generate_code(&self) -> TokenStream2;
}

/// Types implementing this trait can use sub-generators to generate code.
pub trait GenerateCodeUsing {
    /// Returns a reference to the underlying contract.
    fn contract(&self) -> &ir::Contract;

    /// Generates ink! contract code using a sub-generator.
    fn generate_code_using<'a, G>(&'a self) -> TokenStream2
    where
        G: From<&'a ir::Contract> + GenerateCode,
    {
        G::from(self.contract()).generate_code()
    }
}
