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

//! The ink! intermediate representation (IR) and abstractions.
//!
//! This module defines everything the ink! procedural macro needs in order to
//! parse, analyze and generate code for ink! smart contracts.
//!
//! The entry point for every ink! smart contract is the [`Contract`](`crate::ir::Contract`)
//! with its [`Config`](`crate::ir::Config`) provided in the initial invokation at
//! `#[ink::contract(... configuration ...)]`.
//!
//! The ink! IR tries to stay close to the original Rust syntactic structure.
//! All ink! definitions of an ink! smart contract are always defined within
//! a so-called Rust inline modlue (`mod my_module { ... items ... }`).
//! Therefore all ink! definition are found and accessed using the
//! [`ItemMod`](`crate::ir::ItemMod`) data structure.

#![allow(dead_code)]

mod attrs;
mod config;
mod contract;
mod item;
mod item_impl;
mod item_mod;
mod selector;

#[cfg(test)]
use self::attrs::Attribute;

use self::attrs::{
    contains_ink_attributes,
    first_ink_attribute,
    partition_attributes,
    sanitize_attributes,
    AttributeArg,
    AttributeArgKind,
    InkAttribute,
};
pub use self::{
    attrs::Salt,
    config::{
        Config,
        EnvTypes,
    },
    contract::Contract,
    item::{
        Event,
        InkItem,
        Item,
        Storage,
    },
    item_impl::{
        Callable,
        Constructor,
        ImplItem,
        ItemImpl,
        IterConstructors,
        IterMessages,
        Message,
        Receiver,
        Visibility,
    },
    item_mod::{
        ItemMod,
        IterEvents,
        IterImplBlocks,
    },
    selector::Selector,
};
