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

use parity_codec::{Encode, Decode};

use core::marker::PhantomData;

/// Marks types as non-`Copy` and non-`Clone`.
///
/// # Note
///
/// - This marker type is zero-sized for all `T`.
/// - This serves as documentation for crate maintainers
///   as well as utility to enforce this behaviour for types
///   that are using it.
/// - Especially for `Cell` types it is important to make them
///   non-`Copy` and non-`Clone` since that would violate their
///   ownership guarantees over their contract storage slot.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct NonCloneMarker<T>(PhantomData<T>);

impl<T> Default for NonCloneMarker<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}
