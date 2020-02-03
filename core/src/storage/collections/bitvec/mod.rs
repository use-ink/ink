// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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
//! |                     BitBlock 0                    |                     BitBlock 1                    |
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

#[cfg(test)]
mod tests;

mod block;
mod pack;
mod vec;

pub use self::vec::{
    BitVec,
    Iter,
};
pub(self) use self::{
    block::BitBlock,
    pack::BitPack,
};
