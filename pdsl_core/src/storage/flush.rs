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

/// Types that are able to flush their state into the contract storage.
///
/// # Note
///
/// Many types support caching of their state into memory to avoid costly
/// contract storage reads or writes. When execution of a contract is finished
/// or interrupted (e.g. due to calling a remote contract) we have to flush
/// all cached state into the contract storage.
///
/// # Implementation Hints
///
/// Caching types provided by pDSL are `SyncCell` for caching of a single data
/// and `SyncChunk` for caching an array of data.
///
/// All abstractions built upon them that do not have their own caching mechanism
/// shall simply forward flushing to their interiors. Examples for this are
/// `storage::Vec` or `storage::Value`.
pub trait Flush {
	/// Flushes the cached state back to the contract storage, if any.
	fn flush();
}
