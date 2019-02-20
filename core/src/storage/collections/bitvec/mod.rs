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

//! A space-efficient contiguous growable bit array type.
//!
//! Stores its elements in the contract's storage
//! and operates directly on it.
//!
//! # Structure
//!
//! The storage [`BitVec`](struct.BitVec.html) consists of multiple
//! `BitBlock`s each containing 32 `BitPack`s that each consists
//! of `32` bits. So every bit block contains exactly 1024 bits.
//!
//! A graphical visualization is about the following:
//!
//! ```no-compile
//! | bit 0 | ... | bit 32 | ... | bit 0 | ... | bit 32 | bit 0 | ... | bit 32 | ... | bit 0 | ... | bit 32 |
//! |      BitPack 0       | ... |      BitPack 32      |      BitPack 0       | ... |      BitPack 32      |
//! |                     BitBlock 0                    |                     BitBlock 0                    |
//! ```
//!
//! The above pseudo code represents a [`BitVec`](struct.BitVec.html) with 1024 bits.
//!
//! Why the need for `BitBlock`s? They chunk the total set of bits into chunks that
//! are finally stored in the contract storage. So instead of storing each and every bit
//! in a separate storage entry `BitBlock`s exist to bundle them reducing the overall
//! costs and improving efficiency.
//!
//! Besides that a [`BitVec`](struct.BitVec.html) more or less works very similar to a
//! [`Vec`](struct.Vec.html) of `bool`s.

#[cfg(all(test, feature = "test-env"))]
mod tests;

mod block;
mod vec;

pub(in self) use self::block::BitBlock;
pub use self::vec::{
	BitVec,
	Iter,
};
