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
    Iter,
    IterMut,
    Stash as StorageStash,
};
use crate::{
    storage2 as storage,
    storage2::{
        ClearForward,
        KeyPtr,
        PullForward,
        PushForward,
        SaturatingStorage,
        StorageFootprint,
        StorageFootprintOf,
    },
};
use core::iter::{
    Extend,
    FromIterator,
};
// use typenum::{
//     Add1,
//     Unsigned,
// };

impl<T> core::ops::Index<u32> for StorageStash<T>
where
    T: StorageFootprint + PullForward + scale::Decode,
{
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T> core::ops::IndexMut<u32> for StorageStash<T>
where
    T: SaturatingStorage + StorageFootprint + PullForward + scale::Decode,
{
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

impl<'a, T: 'a> IntoIterator for &'a StorageStash<T>
where
    T: StorageFootprint + PullForward + scale::Decode,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: 'a> IntoIterator for &'a mut StorageStash<T>
where
    T: StorageFootprint + SaturatingStorage + PullForward + scale::Decode,
{
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Extend<T> for StorageStash<T>
where
    T: StorageFootprint + SaturatingStorage + PullForward + scale::Codec,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for item in iter {
            self.put(item);
        }
    }
}

impl<T> FromIterator<T> for StorageStash<T>
where
    T: StorageFootprint + SaturatingStorage + PullForward + scale::Codec,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut vec = StorageStash::new();
        vec.extend(iter);
        vec
    }
}

impl<T> core::cmp::PartialEq for StorageStash<T>
where
    T: PartialEq + StorageFootprint + PullForward + scale::Decode,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false
        }
        self.iter().zip(other.iter()).all(|(lhs, rhs)| lhs == rhs)
    }
}

impl<T> core::cmp::Eq for StorageStash<T> where
    T: Eq + StorageFootprint + PullForward + scale::Decode
{
}
