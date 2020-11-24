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

#![allow(dead_code)]

mod attrs;
mod config;
mod contract;
mod idents_lint;
mod ink_test;
mod item;
mod item_impl;
mod item_mod;
mod selector;
mod trait_def;
mod chain_extension;
pub mod utils;

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
    attrs::Namespace,
    config::Config,
    contract::Contract,
    ink_test::InkTest,
    chain_extension::Extension,
    item::{
        Event,
        InkItem,
        Item,
        Storage,
    },
    item_impl::{
        Callable,
        CallableKind,
        CallableWithSelector,
        Constructor,
        ImplItem,
        InputsIter,
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
        IterItemImpls,
    },
    selector::Selector,
    trait_def::{
        InkTrait,
        InkTraitConstructor,
        InkTraitItem,
        InkTraitMessage,
        IterInkTraitItems,
    },
};
