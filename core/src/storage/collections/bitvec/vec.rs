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

use super::{BitBlock};
use crate::{
	storage::{
		self,
		chunk::SyncChunk,
		Allocator,
		alloc::{
			AllocateUsing,
			Initialize,
		},
		Flush,
	},
};

/// A space-efficient contiguous growable bit array type.
#[derive(Debug)]
pub struct BitVec {
	/// The number of bits.
	len: storage::Value<u32>,
	/// The bit blocks.
	blocks: SyncChunk<BitBlock>,
}

impl parity_codec::Encode for BitVec {
	fn encode_to<W: parity_codec::Output>(&self, dest: &mut W) {
		self.len.encode_to(dest);
		self.blocks.encode_to(dest);
	}
}

impl parity_codec::Decode for BitVec {
	fn decode<I: parity_codec::Input>(input: &mut I) -> Option<Self> {
		let len = storage::Value::decode(input)?;
		let blocks = SyncChunk::decode(input)?;
		Some(Self{len, blocks})
	}
}

impl AllocateUsing for BitVec {
	unsafe fn allocate_using<A>(alloc: &mut A) -> Self
	where
		A: Allocator,
	{
		Self {
			len: AllocateUsing::allocate_using(alloc),
			blocks: AllocateUsing::allocate_using(alloc),
		}
	}
}

impl Initialize for BitVec {
	type Args = ();

	fn initialize(&mut self, _: Self::Args) {
		self.len.set(0);
	}
}

impl Flush for BitVec {
	fn flush(&mut self) {
		self.len.flush();
		self.blocks.flush();
	}
}

impl Drop for BitVec {
	fn drop(&mut self) {
		for n in 0..self.len_blocks() {
			self.blocks.clear(n)
		}
	}
}

impl BitVec {
	/// Returns the number of bits.
	pub fn len(&self) -> u32 {
		*self.len.get()
	}

	/// Returns `true` if the vector contains no elements.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns the number of blocks currently in use.
	fn len_blocks(&self) -> u32 {
		BitBlock::required_blocks(self.len())
	}

	/// Returns an immutable reference to the block that
	/// contains the n-th bit or `None` if n is out of bounds.
	fn block(&self, n: u32) -> Option<&BitBlock> {
		if n >= self.len() {
			// Note that this check is extraneous
			// since there is already a check in
			// `self.blocks.get`.
			// However, this better states our intent.
			return None
		}
		self.blocks.get(n / BitBlock::BITS_PER_BLOCK)
	}

	/// Returns a mutable reference to the block that contains
	/// the n-th bit or `None` if n is out of bounds.
	fn block_mut(&mut self, n: u32) -> Option<&mut BitBlock> {
		if n >= self.len() {
			// Note that this check is extraneous
			// since there is already a check in
			// `self.blocks.get`.
			// However, this better states our intent.
			return None
		}
		self.blocks.get_mut(n / BitBlock::BITS_PER_BLOCK)
	}

	/// Returns an immutable reference to the last bit block.
	fn last_block(&self) -> Option<&BitBlock> {
		self.block(self.len() - 1)
	}

	/// Returns a mutable reference to the last bit block.
	fn last_block_mut(&mut self) -> Option<&mut BitBlock> {
		self.block_mut(self.len() - 1)
	}

	/// Returns the first bit of the bit vector or `None` it is empty.
	pub fn first(&self) -> Option<bool> {
		self.get(0)
	}

	/// Returns the last bit of the bit vector or `None` if it is empty.
	pub fn last(&self) -> Option<bool> {
		if self.is_empty() {
			return None
		}
		self.get(self.len() - 1)
	}

	/// Appends a bit to the back of the bit vector.
	pub fn push(&mut self, value: bool) {
		let current_blocks = self.len_blocks();
		let pushed_blocks = BitBlock::required_blocks(self.len() + 1);
		if current_blocks < pushed_blocks {
			self.blocks.set(
				(self.len() + 1) / BitBlock::BITS_PER_BLOCK as u32,
				{
					let mut zeroed = BitBlock::zero();
					zeroed.set(0, value);
					zeroed
				}
			)
		} else {
			let new_latest_idx = self.len();
			self
				.last_block_mut()
				.expect("there must be at least one block at this point")
				.set(new_latest_idx % BitBlock::BITS_PER_BLOCK, value)
		}
		self.len += 1;
	}

	/// Removes the last bit from the bit vector and returns it,
	/// or `None` if the bit vector is empty.
	pub fn pop(&mut self) -> Option<bool> {
		if self.len() == 0 {
			return None
		}
		let current_blocks = self.len_blocks();
		let popped_blocks = BitBlock::required_blocks(self.len() - 1);
		let popped = self
			.last_block()
			.expect("there must be at least one block at this point")
			.get((self.len() - 1) % BitBlock::BITS_PER_BLOCK);
		if popped_blocks < current_blocks {
			self.blocks.clear(
				(self.len() - 1) / BitBlock::BITS_PER_BLOCK as u32,
			)
		}
		// It is safe to not set the last bit back to a default
		// value like `0` since we already declared it to be out
		// of bounds by decreasing the length. By this we can
		// decrease the amount of writes to the storage.
		self.len -= 1;
		Some(popped)
	}

	/// Returns the `n`-th bit of the bit vector.
	///
	/// Returns `None` if `n` is out of bounds.
	pub fn get(&self, n: u32) -> Option<bool> {
		let bit_within_block = n % BitBlock::BITS_PER_BLOCK;
		self
			.block(n)
			.map(|block| block.get(bit_within_block))
	}

	/// Sets the n-th bit of the bit vector.
	///
	/// # Panics
	///
	/// If n is out of bounds.
	pub fn set(&mut self, n: u32, value: bool) {
		let bit_within_block = n % BitBlock::BITS_PER_BLOCK;
		self
			.block_mut(n)
			.map(|block| block.set(bit_within_block, value))
			.expect("n is out of bounds")
	}

	/// Flips the n-th bit of the bit vector.
	///
	/// # Panics
	///
	/// If n is out of bounds.
	pub fn flip(&mut self, n: u32) {
		let bit_within_block = n % BitBlock::BITS_PER_BLOCK;
		self
			.block_mut(n)
			.map(|block| block.flip(bit_within_block))
			.expect("n is out of bounds")
	}

	/// Returns an iterator over all bits of the bit vector.
	///
	/// # Note
	///
	/// Uncontrained iteration in smart contracts is most often bad practice
	/// and should generally be avoided. Consider using this iteration only
	/// in the presence of a constrained context, like `bitvec.iter().take(5)`
	/// or similar.
	pub fn iter(&self) -> Iter {
		Iter::new(self)
	}
}

/// Iterator over the bits of a bit vector.
pub struct Iter<'a> {
	bitvec: &'a BitVec,
	begin: u32,
	end: u32,
}

impl<'a> Iter<'a> {
	fn new(bitvec: &'a BitVec) -> Self {
		Self{
			bitvec,
			begin: 0,
			end: bitvec.len(),
		}
	}
}

impl<'a> Iterator for Iter<'a> {
	type Item = bool;

	fn next(&mut self) -> Option<bool> {
		if self.begin == self.end {
			return None
		}
		let next = self.bitvec.get(self.begin);
		self.begin += 1;
		return next
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = (self.end - self.begin) as usize;
		(remaining, Some(remaining))
	}
}

impl<'a> ExactSizeIterator for Iter<'a> {}

impl<'a> DoubleEndedIterator for Iter<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		debug_assert!(self.begin <= self.end);
		if self.begin == self.end {
			return None
		}
		debug_assert_ne!(self.end, 0);
		self.end -= 1;
		self.bitvec.get(self.end)
	}
}
