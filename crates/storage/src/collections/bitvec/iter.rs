// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

use super::{
    super::extend_lifetime,
    BitRefMut,
    Bits256,
    Bits256BitsIter,
    Bits256BitsIterMut,
    Bitvec as StorageBitvec,
    ChunkRef,
};
use crate::collections::vec::{
    Iter as StorageVecIter,
    IterMut as StorageVecIterMut,
};
use core::cmp::min;

/// Iterator over the bits of a storage bit vector.
#[derive(Debug, Copy, Clone)]
pub struct BitsIter<'a> {
    remaining: u32,
    bits256_iter: Bits256Iter<'a>,
    front_iter: Option<Bits256BitsIter<'a>>,
    back_iter: Option<Bits256BitsIter<'a>>,
}

impl<'a> BitsIter<'a> {
    /// Creates a new iterator yielding the bits of the storage bit vector.
    pub(super) fn new(bitvec: &'a StorageBitvec) -> Self {
        Self {
            remaining: bitvec.len(),
            bits256_iter: bitvec.iter_chunks(),
            front_iter: None,
            back_iter: None,
        }
    }
}

impl<'a> ExactSizeIterator for BitsIter<'a> {}

impl<'a> Iterator for BitsIter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut front_iter) = self.front_iter {
                if let front @ Some(_) = front_iter.next() {
                    self.remaining -= 1;
                    return front
                }
            }
            match self.bits256_iter.next() {
                None => {
                    if let Some(back) = self.back_iter.as_mut()?.next() {
                        self.remaining -= 1;
                        return Some(back)
                    }
                    return None
                }
                Some(ref mut front) => {
                    self.front_iter = Some(unsafe { extend_lifetime(front) }.iter());
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.remaining as usize
    }
}

impl<'a> DoubleEndedIterator for BitsIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut back_iter) = self.back_iter {
                if let back @ Some(_) = back_iter.next_back() {
                    self.remaining -= 1;
                    return back
                }
            }
            match self.bits256_iter.next_back() {
                None => {
                    if let Some(front) = self.front_iter.as_mut()?.next_back() {
                        self.remaining -= 1;
                        return Some(front)
                    }
                    return None
                }
                Some(ref mut back) => {
                    self.back_iter = Some(unsafe { extend_lifetime(back) }.iter());
                }
            }
        }
    }
}

/// Iterator over the bits of a storage bit vector.
#[derive(Debug)]
pub struct BitsIterMut<'a> {
    remaining: u32,
    bits256_iter: Bits256IterMut<'a>,
    front_iter: Option<Bits256BitsIterMut<'a>>,
    back_iter: Option<Bits256BitsIterMut<'a>>,
}

impl<'a> BitsIterMut<'a> {
    /// Creates a new iterator yielding the bits of the storage bit vector.
    pub(super) fn new(bitvec: &'a mut StorageBitvec) -> Self {
        Self {
            remaining: bitvec.len(),
            bits256_iter: bitvec.iter_chunks_mut(),
            front_iter: None,
            back_iter: None,
        }
    }
}

impl<'a> ExactSizeIterator for BitsIterMut<'a> {}

impl<'a> Iterator for BitsIterMut<'a> {
    type Item = BitRefMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut front_iter) = self.front_iter {
                if let front @ Some(_) = front_iter.next() {
                    self.remaining -= 1;
                    return front
                }
            }
            match self.bits256_iter.next() {
                None => {
                    if let Some(back) = self.back_iter.as_mut()?.next() {
                        self.remaining -= 1;
                        return Some(back)
                    }
                    return None
                }
                Some(ref mut front) => {
                    self.front_iter = Some(unsafe { extend_lifetime(front) }.iter_mut());
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.remaining as usize
    }
}

impl<'a> DoubleEndedIterator for BitsIterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut back_iter) = self.back_iter {
                if let back @ Some(_) = back_iter.next_back() {
                    self.remaining -= 1;
                    return back
                }
            }
            match self.bits256_iter.next_back() {
                None => {
                    if let Some(front) = self.front_iter.as_mut()?.next_back() {
                        self.remaining -= 1;
                        return Some(front)
                    }
                    return None
                }
                Some(ref mut back) => {
                    self.back_iter = Some(unsafe { extend_lifetime(back) }.iter_mut());
                }
            }
        }
    }
}

/// Iterator over the 256-bit chunks of a storage bitvector.
#[derive(Debug, Copy, Clone)]
pub struct Bits256Iter<'a> {
    /// The storage vector iterator over the internal 256-bit chunks.
    iter: StorageVecIter<'a, Bits256>,
    /// The remaining bits to be yielded.
    remaining: u32,
}

impl<'a> Bits256Iter<'a> {
    /// Creates a new 256-bit chunks iterator over the given storage bitvector.
    pub(super) fn new(bitvec: &'a StorageBitvec) -> Self {
        Self {
            iter: bitvec.bits.iter(),
            remaining: bitvec.len(),
        }
    }
}

impl<'a> Iterator for Bits256Iter<'a> {
    type Item = ChunkRef<&'a Bits256>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None
        }
        let len = min(256, self.remaining);
        self.remaining = self.remaining.saturating_sub(256);
        self.iter
            .next()
            .map(|bits256| ChunkRef::shared(bits256, len))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        self.iter.count()
    }
}

impl<'a> DoubleEndedIterator for Bits256Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None
        }
        let mut len = self.remaining % 256;
        if len == 0 {
            len = 256;
        }
        self.remaining = self.remaining.saturating_sub(len);
        self.iter
            .next_back()
            .map(|bits256| ChunkRef::shared(bits256, len))
    }
}

impl<'a> ExactSizeIterator for Bits256Iter<'a> {}

/// Iterator over mutable 256-bit chunks of a storage bitvector.
#[derive(Debug)]
pub struct Bits256IterMut<'a> {
    /// The storage vector iterator over the internal mutable 256-bit chunks.
    iter: StorageVecIterMut<'a, Bits256>,
    /// The remaining bits to be yielded.
    remaining: u32,
}

impl<'a> Bits256IterMut<'a> {
    /// Creates a new 256-bit chunks iterator over the given storage bitvector.
    pub(super) fn new(bitvec: &'a mut StorageBitvec) -> Self {
        Self {
            remaining: bitvec.len(),
            iter: bitvec.bits.iter_mut(),
        }
    }
}

impl<'a> Iterator for Bits256IterMut<'a> {
    type Item = ChunkRef<&'a mut Bits256>;

    fn next(&mut self) -> Option<Self::Item> {
        let len = min(256, self.remaining);
        self.remaining = self.remaining.saturating_sub(256);
        self.iter
            .next()
            .map(|bits256| ChunkRef::exclusive(bits256, len))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        self.iter.count()
    }
}

impl<'a> DoubleEndedIterator for Bits256IterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut len = self.remaining % 256;
        self.remaining -= len;
        if len == 0 {
            len = 256;
        }
        self.iter
            .next_back()
            .map(|bits256| ChunkRef::exclusive(bits256, len))
    }
}

impl<'a> ExactSizeIterator for Bits256IterMut<'a> {}
