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
    Iter,
    SmallVec,
};
use crate::{
    lazy::LazyArrayLength,
    traits::PackedLayout,
};
use core::iter::{
    Extend,
    FromIterator,
};

impl<T, N> Drop for SmallVec<T, N>
where
    T: PackedLayout,
    N: LazyArrayLength<T>,
{
    fn drop(&mut self) {
        self.clear_cells()
    }
}

impl<T, N> core::ops::Index<u32> for SmallVec<T, N>
where
    T: PackedLayout,
    N: LazyArrayLength<T>,
{
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        match self.get(index) {
            Some(value) => value,
            None => {
                panic!(
                    "index out of bounds: the len is {} but the index is {}",
                    self.len(),
                    index
                )
            }
        }
    }
}

impl<T, N> core::ops::IndexMut<u32> for SmallVec<T, N>
where
    T: PackedLayout,
    N: LazyArrayLength<T>,
{
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        let len = self.len();
        match self.get_mut(index) {
            Some(value) => value,
            None => {
                panic!(
                    "index out of bounds: the len is {} but the index is {}",
                    len, index
                )
            }
        }
    }
}

impl<'a, T: 'a, N> IntoIterator for &'a SmallVec<T, N>
where
    T: PackedLayout,
    N: LazyArrayLength<T>,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, N> Extend<T> for SmallVec<T, N>
where
    T: PackedLayout,
    N: LazyArrayLength<T>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for item in iter {
            self.push(item)
        }
    }
}

impl<T, N> FromIterator<T> for SmallVec<T, N>
where
    T: PackedLayout,
    N: LazyArrayLength<T>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut vec = SmallVec::new();
        vec.extend(iter);
        vec
    }
}

impl<T, N> core::cmp::PartialEq for SmallVec<T, N>
where
    T: PartialEq + PackedLayout,
    N: LazyArrayLength<T>,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false
        }
        self.iter().zip(other.iter()).all(|(lhs, rhs)| lhs == rhs)
    }
}

impl<T, N> core::cmp::Eq for SmallVec<T, N>
where
    T: Eq + PackedLayout,
    N: LazyArrayLength<T>,
{
}
