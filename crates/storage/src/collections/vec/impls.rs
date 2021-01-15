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

//! Implementation of generic traits that are useful for the storage vector.

use super::{
    Iter,
    IterMut,
    Vec as StorageVec,
};
use crate::traits::PackedLayout;
use core::iter::{
    Extend,
    FromIterator,
};

impl<T> Drop for StorageVec<T>
where
    T: PackedLayout,
{
    fn drop(&mut self) {
        self.clear_cells();
    }
}

impl<T> core::ops::Index<u32> for StorageVec<T>
where
    T: PackedLayout,
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

impl<T> core::ops::IndexMut<u32> for StorageVec<T>
where
    T: PackedLayout,
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

impl<'a, T: 'a> IntoIterator for &'a StorageVec<T>
where
    T: PackedLayout,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: 'a> IntoIterator for &'a mut StorageVec<T>
where
    T: PackedLayout,
{
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Extend<T> for StorageVec<T>
where
    T: PackedLayout,
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

impl<T> FromIterator<T> for StorageVec<T>
where
    T: PackedLayout,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut vec = StorageVec::new();
        vec.extend(iter);
        vec
    }
}

impl<T> core::cmp::PartialEq for StorageVec<T>
where
    T: PartialEq + PackedLayout,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false
        }
        self.iter().zip(other.iter()).all(|(lhs, rhs)| lhs == rhs)
    }
}

impl<T> core::cmp::Eq for StorageVec<T> where T: Eq + PackedLayout {}
