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

//! The ink! intermediate representation (IR) and abstractions.
//!
//! This module defines everything the ink! procedural macro needs in order to
//! parse, analyze and generate code for ink! smart contracts.
//!
//! The entry point for every ink! smart contract is the
//! [`Contract`](`crate::ir::Contract`) with its [`Config`](`crate::ir::Config`) provided
//! in the initial invocation at `#[ink::contract(... configuration ...)]`.
//!
//! The ink! IR tries to stay close to the original Rust syntactic structure.
//! All ink! definitions of an ink! smart contract are always defined within
//! a so-called Rust inline module (`mod my_module { ... items ... }`).
//! Therefore all ink! definition are found and accessed using the
//! [`ItemMod`](`crate::ir::ItemMod`) data structure.

#![doc(
    html_logo_url = "https://use.ink/img/crate-docs/logo.png",
    html_favicon_url = "https://use.ink/crate-docs/favicon.png"
)]

#[macro_use]
mod error;

pub mod ast;
mod ir;
mod literal;

pub use self::{
    ir::{
        blake2b_256,
        marker,
        utils,
        Blake2x256Macro,
        Callable,
        CallableKind,
        CallableWithSelector,
        ChainExtension,
        ChainExtensionMethod,
        Config,
        Constructor,
        Contract,
        Event,
        ExtensionId,
        ImplItem,
        InkItem,
        InkItemTrait,
        InkTest,
        InkTraitDefinition,
        InkTraitItem,
        InkTraitMessage,
        InputsIter,
        IsDocAttribute,
        Item,
        ItemImpl,
        ItemMod,
        IterConstructors,
        IterEvents,
        IterInkTraitItems,
        IterItemImpls,
        IterMessages,
        Message,
        Namespace,
        Receiver,
        Selector,
        SelectorMacro,
        SignatureTopicArg,
        Storage,
        StorageItem,
        Visibility,
    },
    literal::HexLiteral,
};
