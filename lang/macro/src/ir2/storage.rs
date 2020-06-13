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

use proc_macro2::Ident;

/// An ink! storage struct definition.
///
/// Noticed by ink! throught the `#[ink(storage)]` annotation.
///
/// # Note
///
/// An ink! smart contract must have exactly one storage definition.
/// The storage definition must be found in the root of the ink! module.
///
/// # Example
///
/// ```
/// #[ink(storage)]
/// pub struct MyStorage {
///     my_value: bool,
//      counter: u32,
/// }
/// ```
pub struct Storage {
    /// The underlying `struct` Rust item.
    struct_item: syn::ItemStruct,
}

impl Storage {
    /// Returns the identifier of the storage struct.
    pub fn ident(&self) -> &Ident {
        &self.struct_item.ident
    }

    /// Returns an iter yielding all fields of the storage struct.
    pub fn fields(&self) -> syn::punctuated::Iter<syn::Field> {
        self.struct_item.fields.iter()
    }
}
