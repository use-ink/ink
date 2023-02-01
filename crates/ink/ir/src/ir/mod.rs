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

#![allow(dead_code)]

mod attrs;
mod blake2;
mod chain_extension;
mod config;
mod contract;
mod idents_lint;
mod ink_test;
mod item;
mod item_impl;
mod item_mod;
mod selector;
mod storage_item;
mod trait_def;
pub mod utils;

/// Marker types and definitions.
pub mod marker {
    pub use super::selector::{
        SelectorBytes,
        SelectorId,
    };
}

#[cfg(test)]
use self::attrs::Attribute;

use self::attrs::{
    contains_ink_attributes,
    first_ink_attribute,
    partition_attributes,
    sanitize_attributes,
    sanitize_optional_attributes,
    AttributeArg,
    AttributeArgKind,
    AttributeFrag,
    InkAttribute,
};
pub use self::{
    attrs::{
        IsDocAttribute,
        Namespace,
    },
    blake2::{
        blake2b_256,
        Blake2x256Macro,
    },
    chain_extension::{
        ChainExtension,
        ChainExtensionMethod,
        ExtensionId,
    },
    config::Config,
    contract::Contract,
    ink_test::InkTest,
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
    selector::{
        Selector,
        SelectorMacro,
        TraitPrefix,
    },
    storage_item::StorageItem,
    trait_def::{
        InkItemTrait,
        InkTraitDefinition,
        InkTraitItem,
        InkTraitMessage,
        IterInkTraitItems,
    },
};
