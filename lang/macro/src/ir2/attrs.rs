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

use crate::ir2::Selector;

/// Either an ink! specific attribute, or another uninterpreted attribute.
pub enum Attribute {
    /// An ink! specific attribute, e.g. `#[ink(storage)]`.
    Ink(InkAttribute),
    /// Any other attribute.
    ///
    /// This can be a known `#[derive(Debug)]` or a specific attribute of another
    /// crate.
    Other(syn::Attribute),
}

/// Errors that can occur upon parsing ink! attributes.
pub enum Error {
    /// `#[ink()]` or `#[ink]`
    EmptyFlags {
        empty: syn::Attribute,
    },
    /// `#[ink(unknown_flag)]`
    UnknownFlag {
        unknown: syn::Attribute,
    },
    /// `#[ink(selector = true)]`
    InvalidFlag {
        invalid: syn::Attribute,
    },
    /// `#[ink(message, message)]`
    DuplicateFlags {
        duplicate_flags: syn::Attribute,
    },
    /// ```
    /// #[ink(storage)]
    /// #[ink(storage)]
    /// pub struct MyStorage { .. }
    /// ```
    DuplicateAttributes {
        fst: syn::Attribute,
        snd: syn::Attribute,
    },
}

/// An ink! specific attribute.
///
/// # Examples
///
/// An attribute with a simple flag:
/// ```no_compile
/// #[ink(storage)]
/// ```
///
/// An attribute with a parameterized flag:
/// ```no_compile
/// #[ink(selector = "0xDEADBEEF")]
/// ```
///
/// An attribute with multiple flags:
/// ```no_compile
/// #[ink(message, payable, selector = "0xDEADBEEF")]
/// ```
pub struct InkAttribute {
    /// The internal flags of the ink! attribute.
    flags: Vec<AttributeFlag>,
}

impl InkAttribute {
    /// Returns an iterator over the non-empty flags of the ink! attribute.
    ///
    /// # Note
    ///
    /// This yields at least one ink! attribute flag.
    pub fn flags(&self) -> core::slice::Iter<AttributeFlag> {
        self.flags.iter()
    }
}

/// An ink! specific attribute flag.
pub enum AttributeFlag {
    /// `#[ink(storage)]`
    ///
    /// Applied on `struct` or `enum` types in order to flag them for being
    /// the contract's storage definition.
    Storage,
    /// `#[ink(event)]`
    ///
    /// Applied on `struct` types in order to flag them for being an ink! event.
    Event,
    /// `#[ink(topic)]`
    ///
    /// Applied on fields of ink! event types to indicate that they are topics.
    Topic,
    /// `#[ink(message)]`
    ///
    /// Applied on `&self` or `&mut self` methods to flag them for being an ink!
    /// exported message.
    Message,
    /// `#[ink(constructor)]`
    ///
    /// Applied on inherent methods returning `Self` to flag them for being ink!
    /// exported contract constructors.
    Constructor,
    /// `#[ink(payable)]`
    ///
    /// Applied on ink! constructors or messages in order to specify that they
    /// can receive funds from callers.
    Payable,
    /// `#[ink(selector = "0xDEADBEEF")]`
    ///
    /// Applied on ink! constructors or messages to manually control their
    /// selectors.
    Selector(Selector),
    /// `#[ink(salt = "my_salt_message")]`
    ///
    /// Applied on ink! trait implementation blocks to disambiguate other trait
    /// implementation blocks with equal names.
    Salt(Salt),
    /// `#[ink(impl)]`
    ///
    /// This attribute supports a niche case that is rarely needed.
    ///
    /// Can be applied on ink! implementation blocks in order to make ink! aware
    /// of them. This is useful if such an implementation block doesn't contain
    /// any other ink! attributes, so it would be flagged by ink! as a Rust item.
    /// Adding `#[ink(impl)]` on such implementation blocks makes them treated
    /// as ink! implementation blocks thus allowing to access the environment
    /// etc. Note that ink! messages and constructors still need to be explicitely
    /// flagged as such.
    Implementation,
}

/// An ink! salt applicable to a trait implementation block.
pub struct Salt {
    /// The underlying bytes.
    bytes: Vec<u8>,
}

impl Salt {
    /// Returns the salt as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
