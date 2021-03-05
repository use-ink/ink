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
    CallableWithSelector,
    ImplItem,
    ItemImpl,
};
use crate::ir;

/// Iterator yielding all ink! constructor within a source ink!
/// [`ir::ItemImpl`](`crate::ir::ItemImpl`).
pub struct IterConstructors<'a> {
    item_impl: &'a ir::ItemImpl,
    impl_items: core::slice::Iter<'a, ImplItem>,
}

impl<'a> IterConstructors<'a> {
    /// Creates a new ink! messages iterator.
    pub(super) fn new(item_impl: &'a ItemImpl) -> Self {
        Self {
            item_impl,
            impl_items: item_impl.items.iter(),
        }
    }
}

impl<'a> Iterator for IterConstructors<'a> {
    type Item = CallableWithSelector<'a, ir::Constructor>;

    fn next(&mut self) -> Option<Self::Item> {
        'repeat: loop {
            match self.impl_items.next() {
                None => return None,
                Some(impl_item) => {
                    if let Some(constructor) = impl_item.filter_map_constructor() {
                        return Some(CallableWithSelector::new(
                            self.item_impl,
                            constructor,
                        ))
                    }
                    continue 'repeat
                }
            }
        }
    }
}

/// Iterator yielding all ink! messages within a source ink!
/// [`ir::ItemImpl`](`crate::ir::ItemImpl`).
pub struct IterMessages<'a> {
    item_impl: &'a ir::ItemImpl,
    impl_items: core::slice::Iter<'a, ImplItem>,
}

impl<'a> IterMessages<'a> {
    /// Creates a new ink! messages iterator.
    pub(super) fn new(item_impl: &'a ItemImpl) -> Self {
        Self {
            item_impl,
            impl_items: item_impl.items.iter(),
        }
    }
}

impl<'a> Iterator for IterMessages<'a> {
    type Item = CallableWithSelector<'a, ir::Message>;

    fn next(&mut self) -> Option<Self::Item> {
        'repeat: loop {
            match self.impl_items.next() {
                None => return None,
                Some(impl_item) => {
                    if let Some(message) = impl_item.filter_map_message() {
                        return Some(CallableWithSelector::new(self.item_impl, message))
                    }
                    continue 'repeat
                }
            }
        }
    }
}
