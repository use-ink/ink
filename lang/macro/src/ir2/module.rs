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
use proc_macro2::Ident;
use syn::{
    token,
    Attribute,
    Visibility,
};

/// The ink! module.
///
/// This is the root of all ink! smart contracts and is defined similarly to
/// a normal Rust module annotated with `#[ink::contract(version = *)]` attribute.
///
/// It contains ink! specific items as well as normal Rust items.
///
/// # Note
///
/// Structurally the ink! `Module` mirrors an inline Rust module, for example:
/// ```no_compile
/// mod rust_module { ... }
/// ```
/// If the capabilities of an inline Rust module change we have to adjust for that.
pub struct Module {
    attrs: Vec<Attribute>,
    vis: Visibility,
    mod_token: token::Mod,
    ident: Ident,
    brace: token::Brace,
    items: Vec<ir2::Item>,
}

impl Module {
    /// Returns the identifier of the ink! module.
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    /// Returns the storage struct definition for this ink! module.
    ///
    /// # Note
    ///
    /// The storage definition is the struct that has been annotated with
    /// `#[ink(storage)]`. This struct is required to be defined in the root
    /// of the ink! inline module.
    ///
    /// # Panics
    ///
    /// If zero or multiple `#[ink(storage)]` annotated structs were found in
    /// the ink! module. This can be expected to never happen since upon
    /// construction of an ink! module it is asserted that exactly one
    /// `#[ink(storage)]` struct exists.
    pub fn storage(&self) -> &ir2::Storage {
        let mut iter = IterInkItems::new(self)
            .filter_map(|ink_item| ink_item.filter_map_storage_item());
        let storage = iter
            .next()
            .expect("encountered ink! module without a storage struct");
        assert!(
            iter.next().is_none(),
            "encountered multiple storage structs in ink! module"
        );
        storage
    }

    /// Returns the non-ink! item definitions of the ink! inline module.
    ///
    /// Also returns the brace pair that encompasses all ink! definitions.
    pub fn items(&self) -> &[ir2::Item] {
        self.items.as_slice()
    }

    /// Returns an iterator yielding all ink! implementation blocks.
    ///
    /// # Note
    ///
    /// An ink! implementation block can be either an inherent `impl` block
    /// directly defined for the contract's storage struct if it includes at
    /// least one `#[ink(message)]` or `#[ink(constructor)]` annotation, e.g.:
    ///
    /// ```no_compile
    /// #[ink(storage)]
    /// struct MyStorage { ... }
    ///
    /// impl MyStorage {
    ///    #[ink(message)]
    ///    fn my_message(&self) { ... }
    /// }
    /// ```
    ///
    /// Also an implementation block can be defined as a trait implementation
    /// for the ink! storage struct using the `#[ink(extern)]` annotation, e.g.:
    ///
    /// ```no_compile
    /// #[ink(storage)]
    /// struct MyStorage { ... }
    ///
    /// #[ink(extern)]
    /// impl Default for MyStorage {
    ///    #[ink(constructor)]
    ///    fn default() { ... }
    /// }
    /// ```
    pub fn impl_blocks(&self) -> IterImplBlocks {
        IterImplBlocks::new(self)
    }

    /// Returns an iterator yielding all event definitions in this ink! module.
    pub fn events(&self) -> IterEvents {
        IterEvents::new(self)
    }
}

/// Iterator yielding ink! item definitions of the ink! smart contract.
pub struct IterInkItems<'a> {
    items_iter: core::slice::Iter<'a, ir2::Item>,
}

impl<'a> IterInkItems<'a> {
    /// Creates a new ink! module items iterator.
    fn new(ink_module: &'a Module) -> Self {
        Self {
            items_iter: ink_module.items.iter(),
        }
    }
}

impl<'a> Iterator for IterInkItems<'a> {
    type Item = &'a ir2::InkItem;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.items_iter.next() {
                None => return None,
                Some(item) => {
                    if let Some(event) = item.map_ink_item() {
                        return Some(event)
                    }
                    continue 'outer
                }
            }
        }
    }
}

/// Iterator yielding ink! event definition of the ink! smart contract definition.
pub struct IterEvents<'a> {
    items_iter: IterInkItems<'a>,
}

impl<'a> IterEvents<'a> {
    /// Creates a new ink! events iterator.
    fn new(ink_module: &'a Module) -> Self {
        Self {
            items_iter: IterInkItems::new(ink_module),
        }
    }
}

impl<'a> Iterator for IterEvents<'a> {
    type Item = &'a ir2::Event;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.items_iter.next() {
                None => return None,
                Some(ink_item) => {
                    if let Some(event) = ink_item.filter_map_event_item() {
                        return Some(event)
                    }
                    continue 'outer
                }
            }
        }
    }
}

/// Iterator yielding ink! `impl` blocks of the ink! smart contract definition.
pub struct IterImplBlocks<'a> {
    items_iter: IterInkItems<'a>,
}

impl<'a> IterImplBlocks<'a> {
    /// Creates a new ink! implementation blocks iterator.
    fn new(ink_module: &'a Module) -> Self {
        Self {
            items_iter: IterInkItems::new(ink_module),
        }
    }
}

impl<'a> Iterator for IterImplBlocks<'a> {
    type Item = &'a ir2::ImplBlock;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.items_iter.next() {
                None => return None,
                Some(ink_item) => {
                    if let Some(event) = ink_item.filter_map_impl_block() {
                        return Some(event)
                    }
                    continue 'outer
                }
            }
        }
    }
}
