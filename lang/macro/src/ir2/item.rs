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

use crate::ir2;

/// An item in the root of the ink! module.
///
/// This is either an ink! specific item or a normal Rust item.
pub enum Item {
    /// The item is an ink! specific item.
    Ink(InkItem),
    /// The item is a normal Rust item.
    Rust(syn::Item),
}

impl Item {
    /// Returns `true` if `self` is an ink! specific item.
    pub fn is_ink_item(&self) -> bool {
        self.map_ink_item().is_some()
    }

    /// Returns `true` if `self` is an normal Rust item.
    pub fn is_rust_item(&self) -> bool {
        self.map_rust_item().is_some()
    }

    /// Returns `Some` if `self` is an ink! specific item.
    ///
    /// Otherwise, returns `None`.
    pub fn map_ink_item(&self) -> Option<&InkItem> {
        match self {
            Item::Ink(ink_item) => Some(ink_item),
            _ => None,
        }
    }

    /// Returns `Some` if `self` is an ink! specific item.
    ///
    /// Otherwise, returns `None`.
    pub fn map_rust_item(&self) -> Option<&syn::Item> {
        match self {
            Item::Rust(rust_item) => Some(rust_item),
            _ => None,
        }
    }
}

/// An ink! specific item.
pub enum InkItem {
    /// The ink! storage struct definition.
    Storage(ir2::Storage),
    /// An ink! event definition.
    Event(ir2::Event),
    /// An ink! implementation block.
    ImplBlock(ir2::ImplBlock),
}

impl InkItem {
    /// Returns `Some` if `self` is the ink! storage struct definition.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_storage_item(&self) -> Option<&ir2::Storage> {
        match self {
            InkItem::Storage(storage) => Some(storage),
            _ => None,
        }
    }

    /// Returns `true` if the ink! specific item is the storage struct definition.
    pub fn is_storage_item(&self) -> bool {
        self.filter_map_storage_item().is_some()
    }

    /// Returns `Some` if `self` is an ink! event struct definition.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_event_item(&self) -> Option<&ir2::Event> {
        match self {
            InkItem::Event(event) => Some(event),
            _ => None,
        }
    }

    /// Returns `true` if the ink! specific item is an event struct definition.
    pub fn is_event_item(&self) -> bool {
        self.filter_map_event_item().is_some()
    }

    /// Returns `Some` if `self` is an ink! implementation block.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_impl_block(&self) -> Option<&ir2::ImplBlock> {
        match self {
            InkItem::ImplBlock(impl_block) => Some(impl_block),
            _ => None,
        }
    }

    /// Returns `true` if the ink! specific item is an implementation block.
    pub fn is_impl_block(&self) -> bool {
        self.filter_map_impl_block().is_some()
    }
}
