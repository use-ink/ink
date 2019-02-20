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

/// The representation type for a pack.
pub type BitPack = u32;

/// A block of 1024 bits.
#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct BitBlock {
	/// The actual bits.
	packs: [BitPack; Self::COUNT_PACKS as usize]
}

impl BitBlock {
	/// The bits of a single pack.
	const BITS_PER_PACK: u32 = (core::mem::size_of::<BitPack>() * 8) as u32;

	/// The number of packs of bits.
	const COUNT_PACKS: u32 = 32;

	/// The number of bits of a bit block.
	pub const BITS_PER_BLOCK: u32 = Self::BITS_PER_PACK * Self::COUNT_PACKS;

	/// Creates a new zeroed bit block.
	pub fn zero() -> Self {
		Self {
			packs: [0x0; Self::COUNT_PACKS as usize]
		}
	}

	/// Returns the number of required blocks for the given number of bits.
	pub fn required_blocks(n: u32) -> u32 {
		if n == 0 {
			return 0
		}
		1 + ((n - 1) / Self::BITS_PER_BLOCK)
	}

	/// Returns the value of the n-th bit.
	///
	/// # Panics
	///
	/// If n is out of bounds.
	pub fn get(&self, n: u32) -> bool {
		if n >= Self::BITS_PER_BLOCK {
			panic!("bit access out of bounds")
		}
		let pack = self.packs[(n / Self::COUNT_PACKS) as usize];
		let bit = pack & (0x1 << (n % Self::BITS_PER_PACK));
		bit != 0
	}

	/// Sets the value of the n-th bit.
	///
	/// # Panics
	///
	/// If n is out of bounds.
	pub fn set(&mut self, n: u32, value: bool) {
		if n >= Self::BITS_PER_BLOCK {
			panic!("bit access out of bounds")
		}
		let pack = &mut self.packs[(n / Self::COUNT_PACKS) as usize];
		let pack_n = n % Self::BITS_PER_PACK;
		if value {
			*pack |= 0x1 << pack_n;
		} else {
			*pack &= !(0x1 << pack_n);
		}
	}

	/// Flips the value of the n-th bit.
	///
	/// # Panics
	///
	/// If n is out of bounds.
	pub fn flip(&mut self, n: u32) {
		if n >= Self::BITS_PER_BLOCK {
			panic!("bit access out of bounds")
		}
		let pack = &mut self.packs[(n / Self::COUNT_PACKS) as usize];
		*pack ^= 0x1 << (n % Self::BITS_PER_PACK);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn max_valid_index() -> u32 {
		BitBlock::BITS_PER_BLOCK - 1
	}

	#[test]
	fn req_blocks() {
		assert_eq!(BitBlock::required_blocks(0), 0);
		assert_eq!(BitBlock::required_blocks(1), 1);
		assert_eq!(BitBlock::required_blocks(2), 1);
		assert_eq!(BitBlock::required_blocks(BitBlock::BITS_PER_BLOCK / 2), 1);
		assert_eq!(BitBlock::required_blocks(BitBlock::BITS_PER_BLOCK - 1), 1);
		assert_eq!(BitBlock::required_blocks(BitBlock::BITS_PER_BLOCK), 1);
		assert_eq!(BitBlock::required_blocks(BitBlock::BITS_PER_BLOCK + 1), 2);
		assert_eq!(BitBlock::required_blocks((2 * BitBlock::BITS_PER_BLOCK) - 1), 2);
		assert_eq!(BitBlock::required_blocks(2 * BitBlock::BITS_PER_BLOCK), 2);
		assert_eq!(BitBlock::required_blocks((2 * BitBlock::BITS_PER_BLOCK) + 1), 3);
	}

	#[test]
	fn zero() {
		let zero_block = BitBlock::zero();
		for n in 0..BitBlock::BITS_PER_BLOCK {
			assert_eq!(zero_block.get(n), false)
		}
	}

	#[test]
	#[should_panic]
	fn get_out_of_bounds() {
		let block = BitBlock::zero();
		block.get(BitBlock::BITS_PER_BLOCK);
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
		block.set(BitBlock::BITS_PER_BLOCK, true);
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
		block.flip(BitBlock::BITS_PER_BLOCK);
	}
}
