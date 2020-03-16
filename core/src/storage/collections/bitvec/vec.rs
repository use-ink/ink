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

#[cfg(feature = "ink-generate-abi")]
use ink_abi::{
    HasLayout,
    LayoutField,
    LayoutStruct,
    StorageLayout,
};
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

use super::BitBlock;
use crate::storage::{
    self,
    alloc::{
        Allocate,
        AllocateUsing,
        Initialize,
    },
    chunk::SyncChunk,
    Flush,
};

/// A space-efficient contiguous growable bit array type.
#[derive(Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct BitVec {
    /// The number of bits.
    len: storage::Value<u32>,
    /// The bit blocks.
    blocks: SyncChunk<BitBlock>,
}

#[cfg(feature = "ink-generate-abi")]
impl HasLayout for BitVec {
    fn layout(&self) -> StorageLayout {
        LayoutStruct::new(
            Self::meta_type(),
            vec![
                LayoutField::of("len", &self.len),
                LayoutField::of("blocks", &self.blocks),
            ],
        )
        .into()
    }
}

impl scale::Encode for BitVec {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.len.encode_to(dest);
        self.blocks.encode_to(dest);
    }
}

impl scale::Decode for BitVec {
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        let len = storage::Value::decode(input)?;
        let blocks = SyncChunk::decode(input)?;
        Ok(Self { len, blocks })
    }
}

impl AllocateUsing for BitVec {
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            len: AllocateUsing::allocate_using(alloc),
            blocks: AllocateUsing::allocate_using(alloc),
        }
    }
}

impl Initialize for BitVec {
    type Args = ();

    #[inline(always)]
    fn default_value() -> Option<Self::Args> {
        Some(())
    }

    #[inline]
    fn initialize(&mut self, _: Self::Args) {
        self.len.set(0);
    }
}

impl Flush for BitVec {
    #[inline]
    fn flush(&mut self) {
        self.len.flush();
        self.blocks.flush();
    }
}

impl Drop for BitVec {
    #[inline]
    fn drop(&mut self) {
        for n in 0..self.len_blocks() {
            self.blocks.clear(n)
        }
    }
}

impl Extend<bool> for BitVec {
    fn extend<T: IntoIterator<Item = bool>>(&mut self, iter: T) {
        for b in iter {
            self.push(b)
        }
    }
}

impl<'a> Extend<&'a bool> for BitVec {
    fn extend<T: IntoIterator<Item = &'a bool>>(&mut self, iter: T) {
        self.extend(iter.into_iter().copied())
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
        self.blocks.get(n / BitBlock::BITS)
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
        self.blocks.get_mut(n / BitBlock::BITS)
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
            self.blocks.set((self.len() + 1) / BitBlock::BITS, {
                let mut zeroed = BitBlock::zero();
                zeroed.set(0, value);
                zeroed
            })
        } else {
            let new_latest_idx = self.len();
            self.last_block_mut()
                .expect("there must be at least one block at this point")
                .set(new_latest_idx % BitBlock::BITS, value)
        }
        self.len += 1;
    }

    /// Removes the last bit from the bit vector and returns it,
    /// or `None` if the bit vector is empty.
    pub fn pop(&mut self) -> Option<bool> {
        if self.is_empty() {
            return None
        }
        let len = self.len();
        let current_blocks = self.len_blocks();
        let popped_blocks = BitBlock::required_blocks(len - 1);
        let popped = self
			.last_block()
			.expect("we already checked that len is greater than 1 so there must be at least one block; qed")
			.get((len - 1) % BitBlock::BITS);
        if popped_blocks < current_blocks {
            // Remove last bit block.
            self.blocks.clear((len - 1) / BitBlock::BITS)
        } else {
            // Set last bit in last bit block to false.
            self.last_block_mut()
                .expect(
                    "since we have the same amount of blocks we have at least one; qed",
                )
                .set((len - 1) % BitBlock::BITS, false)
        }
        self.len -= 1;
        Some(popped)
    }

    /// Returns the `n`-th bit of the bit vector.
    ///
    /// Returns `None` if `n` is out of bounds.
    pub fn get(&self, n: u32) -> Option<bool> {
        let bit_within_block = n % BitBlock::BITS;
        self.block(n).map(|block| block.get(bit_within_block))
    }

    /// Sets the n-th bit of the bit vector.
    ///
    /// # Panics
    ///
    /// If n is out of bounds.
    pub fn set(&mut self, n: u32, value: bool) {
        let bit_within_block = n % BitBlock::BITS;
        self.block_mut(n)
            .map(|block| block.set(bit_within_block, value))
            .expect("n is out of bounds")
    }

    /// Flips the n-th bit of the bit vector.
    ///
    /// # Panics
    ///
    /// If n is out of bounds.
    pub fn flip(&mut self, n: u32) {
        let bit_within_block = n % BitBlock::BITS;
        self.block_mut(n)
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

    /// Returns an iterator over all bit blocks of `self`.
    ///
    /// # Note
    ///
    /// This is meant to be an internal API that shall not be
    /// exposed to the outside.
    fn iter_blocks(&self) -> BlockIter {
        BlockIter::new(self)
    }

    /// Returns the position of the first set bit in `self` if any.
    ///
    /// # Note
    ///
    /// This is a lot more efficient than to naively iterate
    /// through all bits of the bit vector.
    ///
    /// # Complexity
    ///
    /// The worst-case time complexity of this procedure is
    /// linear with respect to the length of `self`.
    pub fn first_set_position(&self) -> Option<u32> {
        for (n, block) in self.iter_blocks().enumerate() {
            if let Some(pos) = block.first_set_position() {
                return Some(n as u32 * BitBlock::BITS + pos)
            }
        }
        None
    }
}

/// Iterator over the bit blocks of a bit vector.
///
/// # Note
///
/// This is an internal iterator that should not be exposed
/// to the outside.
struct BlockIter<'a> {
    bitvec: &'a BitVec,
    begin: u32,
    end: u32,
}

impl<'a> BlockIter<'a> {
    fn new(bitvec: &'a BitVec) -> Self {
        Self {
            bitvec,
            begin: 0,
            end: BitBlock::required_blocks(bitvec.len()),
        }
    }
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = &'a BitBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if self.begin == self.end {
            return None
        }
        let next = self.bitvec.blocks.get(self.begin).expect(
            "block are allocated contigeously in storage\
             ; so there has to be a block here; qed",
        );
        self.begin += 1;
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.end - self.begin) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for BlockIter<'a> {}

impl<'a> DoubleEndedIterator for BlockIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.begin == self.end {
            return None
        }
        debug_assert_ne!(self.end, 0);
        self.end -= 1;
        let block = self.bitvec.blocks.get(self.end).expect(
            "block are allocated contigeously in storage\
             ; so there has to be a block here; qed",
        );
        Some(block)
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
        Self {
            bitvec,
            begin: 0,
            end: bitvec.len(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.begin == self.end {
            return None
        }
        let next = self.bitvec.get(self.begin);
        self.begin += 1;
        next
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
