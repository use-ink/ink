// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

//! Custom derive for `ink_storage` traits.
//!
//! This crate provides helpers to define your very own custom storage data
//! structures that work along the `ink_storage` data structures.

extern crate proc_macro;

mod storable;

#[cfg(test)]
mod tests;

use self::storable::storable_derive;
synstructure::decl_derive!(
    [Storable] =>
    /// Derives `ink_storage`'s `Storable` trait for the given `struct`, `enum` or `union`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink_primitives::traits::Storable;
    ///
    /// #[derive(Storable)]
    /// struct NamedFields {
    ///     a: u32,
    ///     b: [u32; 1],
    /// }
    ///
    /// let value = <NamedFields as Storable>::decode(&mut &[123, 123][..]);
    /// ```
    storable_derive
);
