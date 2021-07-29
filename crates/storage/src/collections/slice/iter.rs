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

use crate::collections::slice::ContiguousStorage;
use std::ops::Range;

pub struct IterMut<'a, T>
where
    T: ContiguousStorage,
{
    pub(crate) index: u32,
    pub(crate) range: Range<u32>,
    pub(crate) backing_storage: &'a T,
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: ContiguousStorage,
{
    type Item = &'a mut <T as ContiguousStorage>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.range.contains(&self.index) {
            return None
        }

        let item = unsafe { self.backing_storage.get_mut(self.index) };
        self.index += 1;
        item
    }
}

pub struct Iter<'a, T>
where
    T: ContiguousStorage,
{
    pub(crate) index: u32,
    pub(crate) range: Range<u32>,
    pub(crate) backing_storage: &'a T,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: ContiguousStorage,
{
    type Item = &'a <T as ContiguousStorage>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.range.contains(&self.index) {
            return None
        }

        let item = self.backing_storage.get(self.index);
        self.index += 1;
        item
    }
}
