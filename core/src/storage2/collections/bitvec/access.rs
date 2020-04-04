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
    Index256,
};

/// A mutable bit access for operating on a single bit within a 256-bit pack.
#[derive(Debug)]
pub struct BitAccess<'a> {
    /// The queried pack of 256 bits.
    bits: &'a mut Bits256,
    /// The bit position witihn the queried bit pack.
    at: u8,
}

impl<'a> BitAccess<'a> {
    /// Creates a new bit access for the indexed bit within the 256-bit pack.
    pub(super) fn new(bits: &'a mut Bits256, at: Index256) -> Self {
        Self { bits, at }
    }

    /// Returns the value of the indexed bit.
    ///
    /// # Note
    ///
    /// - If 0: returns `false`
    /// - If 1: returns `true`
    pub fn get(&self) -> bool {
        self.bits.get(self.at)
    }

    /// Sets the value of the indexed bit to the given new value.
    pub fn set_to(&mut self, new_value: bool) {
        self.bits.set_to(self.at, new_value)
    }

    /// Sets the indexed bit to `1` (true).
    pub fn set(&mut self) {
        self.bits.set(self.at)
    }

    /// Resets the indexed bit to `0` (false).
    pub fn reset(&mut self) {
        self.bits.set(self.at)
    }

    /// Flips the indexed bit.
    pub fn flip(&mut self) {
        self.bits.flip(self.at)
    }

    /// Computes bitwise XOR for the indexed bit and `rhs`.
    pub fn xor(&mut self, rhs: bool) {
        self.bits.xor(self.at, rhs)
    }

    /// Computes bitwise AND for the indexed bit and `rhs`.
    pub fn and(&mut self, rhs: bool) {
        self.bits.and(self.at, rhs)
    }

    /// Computes bitwise OR for the indexed bit and `rhs`.
    pub fn or(&mut self, rhs: bool) {
        self.bits.or(self.at, rhs)
    }
}
