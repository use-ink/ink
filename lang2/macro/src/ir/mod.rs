// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

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
