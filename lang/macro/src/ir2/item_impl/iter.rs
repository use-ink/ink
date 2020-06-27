// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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
    ItemImpl,
    ImplBlockItem,
};
use crate::ir2;

/// Iterator yielding constructors of the ink! smart contract definition.
pub struct IterConstructors<'a> {
    impl_items: core::slice::Iter<'a, ImplBlockItem>,
}

impl<'a> From<core::slice::Iter<'a, ImplBlockItem>> for IterConstructors<'a> {
    fn from(iter: core::slice::Iter<'a, ImplBlockItem>) -> Self {
        Self { impl_items: iter }
    }
}

impl<'a> IterConstructors<'a> {
    /// Creates a new ink! messages iterator.
    pub(super) fn new(impl_block: &'a ItemImpl) -> Self {
        Self {
            impl_items: impl_block.items.iter(),
        }
    }
}

impl<'a> Iterator for IterConstructors<'a> {
    type Item = &'a ir2::Constructor;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.impl_items.next() {
                None => return None,
                Some(impl_item) => {
                    if let Some(constructor) = impl_item.filter_map_constructor() {
                        return Some(constructor)
                    }
                    continue 'outer
                }
            }
        }
    }
}

/// Iterator yielding messages of the ink! smart contract definition.
pub struct IterMessages<'a> {
    impl_items: core::slice::Iter<'a, ImplBlockItem>,
}

impl<'a> From<core::slice::Iter<'a, ImplBlockItem>> for IterMessages<'a> {
    fn from(iter: core::slice::Iter<'a, ImplBlockItem>) -> Self {
        Self { impl_items: iter }
    }
}

impl<'a> IterMessages<'a> {
    /// Creates a new ink! messages iterator.
    pub(super) fn new(impl_block: &'a ItemImpl) -> Self {
        Self {
            impl_items: impl_block.items.iter(),
        }
    }
}

impl<'a> Iterator for IterMessages<'a> {
    type Item = &'a ir2::Message;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.impl_items.next() {
                None => return None,
                Some(impl_item) => {
                    if let Some(message) = impl_item.filter_map_message() {
                        return Some(message)
                    }
                    continue 'outer
                }
            }
        }
    }
}
