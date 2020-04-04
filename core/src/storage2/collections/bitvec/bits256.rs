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
    Index256,
    Index64,
    Bits64,
};
use crate::storage2::{
    pull_single_cell,
    PullAt,
    PushAt,
};
use ink_primitives::Key;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct Bits256 {
    bits: [Bits64; 4],
}

impl Default for Bits256 {
    fn default() -> Self {
        Self {
            bits: Default::default(),
        }
    }
}

impl Bits256 {
    fn bits_at(&self, index: Index256) -> (&u64, Index64) {
        (&self.bits[(index / 64) as usize], index % 64)
    }

    fn bits_at_mut(&mut self, index: Index256) -> (&mut u64, Index64) {
        (&mut self.bits[(index / 64) as usize], index % 64)
    }

    pub fn get(&self, at: Index256) -> bool {
        let (bits64, pos64) = self.bits_at(at);
        match bits64 & (0x01 << pos64) {
            0 => false,
            1 => true,
            _ => unreachable!(),
        }
    }

    pub fn set_to(&mut self, at: Index256, new_value: bool) {
        if new_value {
            self.set(at)
        } else {
            self.reset(at)
        }
    }

    pub fn flip(&mut self, at: Index256) {
        self.xor(at, true)
    }

    pub fn set(&mut self, at: Index256) {
        self.or(at, true)
    }

    pub fn reset(&mut self, at: Index256) {
        self.and(at, false)
    }

    fn op_at_with<F>(&mut self, at: Index256, rhs: bool, op: F)
    where
        F: FnOnce(&mut Bits64, Bits64),
    {
        let (bits64, pos64) = self.bits_at_mut(at);
        let rhs = (rhs as u64) << (63 - pos64);
        op(bits64, rhs);
    }

    pub fn and(&mut self, at: Index256, rhs: bool) {
        self.op_at_with(at, rhs, |bits64, rhs| *bits64 &= rhs)
    }

    pub fn or(&mut self, at: Index256, rhs: bool) {
        self.op_at_with(at, rhs, |bits64, rhs| *bits64 |= rhs)
    }

    pub fn xor(&mut self, at: Index256, rhs: bool) {
        self.op_at_with(at, rhs, |bits64, rhs| *bits64 ^= rhs)
    }
}

impl PullAt for Bits256 {
    fn pull_at(at: Key) -> Self {
        pull_single_cell(at)
    }
}

impl PushAt for Bits256 {
    fn push_at(&self, at: Key) {
        crate::env::set_contract_storage(at, self)
    }
}
