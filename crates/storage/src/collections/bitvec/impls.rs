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
    BitsIter,
    Bitvec as StorageBitvec,
};
use core::iter::FromIterator;

impl Default for StorageBitvec {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for StorageBitvec {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false
        }
        self.bits.eq(&other.bits)
    }
}

impl Eq for StorageBitvec {}

impl Extend<bool> for StorageBitvec {
    fn extend<T: IntoIterator<Item = bool>>(&mut self, iter: T) {
        for value in iter {
            self.push(value)
        }
    }
}

impl<'a> Extend<&'a bool> for StorageBitvec {
    fn extend<T: IntoIterator<Item = &'a bool>>(&mut self, iter: T) {
        for value in iter {
            self.push(*value)
        }
    }
}

impl FromIterator<bool> for StorageBitvec {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        let mut bitvec = Self::default();
        bitvec.extend(iter);
        bitvec
    }
}

impl<'a> FromIterator<&'a bool> for StorageBitvec {
    fn from_iter<T: IntoIterator<Item = &'a bool>>(iter: T) -> Self {
        <Self as FromIterator<bool>>::from_iter(iter.into_iter().copied())
    }
}

impl<'a> IntoIterator for &'a StorageBitvec {
    type Item = bool;
    type IntoIter = BitsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.bits()
    }
}
