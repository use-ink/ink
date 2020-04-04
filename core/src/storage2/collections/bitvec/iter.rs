// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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
    Bits256,
    Bitvec as StorageBitvec,
};
use crate::storage2::Pack;
use core::cmp::min;

/// Iterator over the bits of a storage bit vector.
pub struct Bits<'a> {
    bitvec: &'a StorageBitvec,
    bits256_id: u32,
    bits256: Option<&'a Bits256>,
    bits256_len: u32,
    bit: u32,
}

impl<'a> Bits<'a> {
    /// Creates a new iterator yielding the bits of the storage bit vector.
    pub(super) fn new(bitvec: &'a StorageBitvec) -> Self {
        Self {
            bitvec,
            bits256_id: 0,
            bits256: None,
            bits256_len: 0,
            bit: 0,
        }
    }
}

impl<'a> Iterator for Bits<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(bits256) = self.bits256 {
                if self.bit == self.bits256_len {
                    self.bits256 = None;
                    continue
                }
                let value = bits256.get(self.bit as u8);
                self.bit += 1;
                return Some(value)
            } else {
                if (self.bits256_id * 256) as u64 == self.bitvec.capacity() {
                    return None
                }
                self.bits256 = Some(
                    self.bitvec
                        .bits
                        .get(self.bits256_id)
                        .map(Pack::as_inner)
                        .expect("id is within bounds"),
                );
                self.bits256_len = min(
                    256,
                    self.bitvec
                        .capacity()
                        .saturating_sub((self.bits256_id * 256) as u64),
                ) as u32;
                self.bit = 0;
                self.bits256_id += 1;
                continue
            }
        }
    }
}
