// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

//! This module provides
//!
//! - Data structures for ink! IR and ink! parameters
//! - Parsing procedures for Rust and ink! code
//! - Conversion routines from rust AST to ink! IR

mod data;
mod into_hir;
mod params;
pub mod utils;

#[cfg(test)]
mod tests;

pub use self::{
    data::{
        Contract,
        FnArg,
        Function,
        FunctionKind,
        FunctionSelector,
        IdentType,
        InkItem,
        Item,
        ItemEvent,
        ItemImpl,
        ItemStorage,
        KindConstructor,
        KindMessage,
        Marker,
        MetaInfo,
        MetaTypes,
        MetaVersion,
        RustItem,
        Signature,
        SimpleMarker,
    },
    params::{
        MetaParam,
        ParamTypes,
        ParamVersion,
        Params,
    },
};
