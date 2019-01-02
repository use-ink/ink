// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

//! Setup contract storage underlying an entity.

/// Types implementing this trait are initializable on the contract storage.
///
/// # Note
///
/// Some types require special initialization routines on the contract storage
/// upon creation to properly operate on it.
pub trait Setup {
	/// Setup contract storage underlying to `self`.
	///
	/// This initializes contract storage used by the entity
	/// to whatever state it expects to be for operating on it.
	///
	/// # Note
	///
	/// This should be executed only once per instance.
	fn setup(&mut self);
}
