// Copyright (C) Use Ink (UK) Ltd.
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
mod config;
mod contract;
mod event;
mod idents_lint;
mod ink_test;
mod item;
mod item_impl;
mod item_mod;
mod selector;
mod sha3;
mod storage_item;
mod trait_def;
pub mod utils;

const CFG_IDENT: &str = "cfg";

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
    AttributeArg,
    AttributeArgKind,
    AttributeFrag,
    InkAttribute,
    contains_ink_attributes,
    first_ink_attribute,
    partition_attributes,
    sanitize_attributes,
    sanitize_optional_attributes,
};
pub use self::{
    attrs::{
        IsDocAttribute,
        Namespace,
    },
    blake2::{
        Blake2x256Macro,
        blake2b_256,
    },
    config::Config,
    contract::Contract,
    event::{
        Event,
        EventConfig,
        SignatureTopic,
    },
    ink_test::InkTest,
    item::{
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
