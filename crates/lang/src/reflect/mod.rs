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

//! Definitions and utilities for ink! smart contract static reflection.
//!
//! # Note
//!
//! The ink! smart contract codegen uses these reflections in order to
//! structure, solidify and manage the generated code.
//!
//! However, the definitions in this module might be useful to ink! smart
//! contract authors as well as they allow to inspect compile time information
//! about the ink! smart contract at hand.

mod contract;
mod dispatch;
mod event;
mod trait_def;

pub use self::{
    contract::{
        ContractEnv,
        ContractName,
        ContractReference,
    },
    dispatch::{
        ContractAmountDispatchables,
        ContractConstructorDecoder,
        ContractDispatchableConstructors,
        ContractDispatchableMessages,
        ContractMessageDecoder,
        DecodeDispatch,
        DispatchError,
        DispatchableConstructorInfo,
        DispatchableMessageInfo,
        ExecuteDispatchable,
    },
    event::{
        ContractEventBase,
        ContractEvent,
    },
    trait_def::{
        TraitDefinitionRegistry,
        TraitInfo,
        TraitMessageInfo,
    },
};
