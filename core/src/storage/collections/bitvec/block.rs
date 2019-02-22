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

use super::{BitPack};
use parity_codec::{Encode, Decode};

/// A block of 1024 bits.
#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct BitBlock {
	/// The underlying bit packs.
	packs: [BitPack; Self::PACKS as usize],
}

/// Error indicating an invalid bit pack index.
#[derive(Debug, Copy, Clone)]
struct InvalidBitBlockIndex;

/// Result type when working with bit packs.
type BitBlockResult<T> = core::result::Result<T, InvalidBitBlockIndex>;

impl BitBlock {
	/// The number of bit packs.
	const PACKS: u32 = 32;

	/// The number of bits of a bit block.
	pub const BITS: u32 = BitPack::BITS * Self::PACKS;

	/// Creates a new zeroed bit block.
	pub const fn zero() -> Self {
		Self {
			packs: [BitPack::new(0x0); Self::PACKS as usize]
		}
	}

	/// Returns the number of required blocks for the given number of bits.
	pub fn required_blocks(n: u32) -> u32 {
		if n == 0 {
			return 0
		}
		1 + ((n - 1) / Self::BITS)
	}

	/// Checks if `n` is within bounds of a bit block and returns it if so.
	fn validate_index(n: u32) -> BitBlockResult<u32> {
		if n >= Self::BITS {
			return Err(InvalidBitBlockIndex)
		}
		Ok(n)
	}

	/// Returns an immutable reference to the associated
	/// bit pack and the bit position within the bit pack
	/// for the given bit index.
	///
	/// # Errors
	///
	/// Returns an error if the given bit index is out of bounds.
	fn pack_and_pos(&self, n: u32) -> BitBlockResult<(&BitPack, u32)> {
		if n >= Self::BITS {
			return Err(InvalidBitBlockIndex)
		}
		Ok((&self.packs[(n / Self::PACKS) as usize], n % Self::PACKS))
	}

	/// Returns a mutable reference to the associated
	/// bit pack and the bit position within the bit pack
	/// for the given bit index.
	///
	/// # Errors
	///
	/// Returns an error if the given bit index is out of bounds.
	fn pack_and_pos_mut(&mut self, n: u32) -> BitBlockResult<(&mut BitPack, u32)> {
		if n >= Self::BITS {
			return Err(InvalidBitBlockIndex)
		}
		Ok((&mut self.packs[(n / Self::PACKS) as usize], n % Self::PACKS))
	}

	/// Returns the value of the n-th bit.
	///
	/// # Panics
	///
	/// If n is out of bounds.
	pub fn get(&self, n: u32) -> bool {
		self.pack_and_pos(n)
			.map(|(pack, pos)| {
				pack.get(pos)
			})
			.unwrap()
	}

	/// Sets the value of the n-th bit.
	///
	/// # Panics
	///
	/// If n is out of bounds.
	pub fn set(&mut self, n: u32, value: bool) {
		self.pack_and_pos_mut(n)
			.map(|(pack, pos)| {
				pack.set(pos, value)
			})
			.unwrap()
	}

	/// Flips the value of the n-th bit.
	///
	/// # Panics
	///
	/// If n is out of bounds.
	pub fn flip(&mut self, n: u32) {
		self.pack_and_pos_mut(n)
			.map(|(pack, pos)| {
				pack.flip(pos)
			})
			.unwrap()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Returns the maximum valid index of a bit block.
	fn max_valid_index() -> u32 {
		BitBlock::BITS - 1
	}

	#[test]
	fn req_blocks() {
		assert_eq!(BitBlock::required_blocks(0), 0);
		assert_eq!(BitBlock::required_blocks(1), 1);
		assert_eq!(BitBlock::required_blocks(2), 1);
		assert_eq!(BitBlock::required_blocks(BitBlock::BITS / 2), 1);
		assert_eq!(BitBlock::required_blocks(BitBlock::BITS - 1), 1);
		assert_eq!(BitBlock::required_blocks(BitBlock::BITS), 1);
		assert_eq!(BitBlock::required_blocks(BitBlock::BITS + 1), 2);
		assert_eq!(BitBlock::required_blocks((2 * BitBlock::BITS) - 1), 2);
		assert_eq!(BitBlock::required_blocks(2 * BitBlock::BITS), 2);
		assert_eq!(BitBlock::required_blocks((2 * BitBlock::BITS) + 1), 3);
	}

	#[test]
	fn zero() {
		let zero_block = BitBlock::zero();
		for n in 0..BitBlock::BITS {
			assert_eq!(zero_block.get(n), false)
		}
	}

	#[test]
	#[should_panic]
	fn get_out_of_bounds() {
		let block = BitBlock::zero();
		block.get(BitBlock::BITS);
	}

	#[test]
	fn set() {
		let mut block = BitBlock::zero();
		block.set(0, true);
		assert_eq!(block.get(0), true);
		block.set(42, false);
		assert_eq!(block.get(42), false);
		block.set(max_valid_index(), true);
		assert_eq!(block.get(max_valid_index()), true);
		block.set(max_valid_index(), false);
		assert_eq!(block.get(max_valid_index()), false);
	}

	#[test]
	#[should_panic]
	fn set_out_of_bounds() {
		let mut block = BitBlock::zero();
		block.set(BitBlock::BITS, true);
	}

	#[test]
	fn flip() {
		let mut block = BitBlock::zero();
		block.flip(0);
		assert_eq!(block.get(0), true);
		block.flip(0);
		assert_eq!(block.get(0), false);
		block.flip(max_valid_index());
		assert_eq!(block.get(max_valid_index()), true);
	}

	#[test]
	#[should_panic]
	fn flip_out_of_bounds() {
		let mut block = BitBlock::zero();
		block.flip(BitBlock::BITS);
	}
}
