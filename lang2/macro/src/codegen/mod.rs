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

mod abi;
mod contract;
mod dispatch;
mod env_types;
mod events;
mod storage;

use crate::ir::Contract;
use proc_macro2::TokenStream as TokenStream2;

/// Types implementing this trait are code generators for the ink! language.
pub trait GenerateCode {
    /// Generates ink! contract code.
    fn generate_code(&self) -> TokenStream2;
}

/// Types implementing this trait can use sub-generators to generate code.
pub trait GenerateCodeUsing {
    /// Returns a reference to the underlying contract.
    fn contract(&self) -> &Contract;

    /// Generates ink! contract code using a sub-generator.
    fn generate_code_using<'a, G>(&'a self) -> TokenStream2
    where
        G: From<&'a Contract> + GenerateCode,
    {
        // crate::codegen::generate_code::<G>(self.contract())
        G::from(self.contract()).generate_code()
    }
}
