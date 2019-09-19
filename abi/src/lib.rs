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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod layout;
mod specs;

#[cfg(feature = "derive")]
pub use ink_abi_derive::HasLayout;

pub use self::{
    layout::{
        HasLayout,
        LayoutField,
        LayoutKey,
        LayoutRange,
        LayoutStruct,
        StorageLayout,
    },
    specs::{
        ConstructorSpec,
        ConstructorSpecBuilder,
        ContractSpec,
        ContractSpecBuilder,
        DisplayName,
        EventParamSpec,
        EventParamSpecBuilder,
        EventSpec,
        EventSpecBuilder,
        MessageParamSpec,
        MessageParamSpecBuilder,
        MessageSpec,
        MessageSpecBuilder,
        ReturnTypeSpec,
        TypeSpec,
    },
};

use serde::Serialize;
use type_metadata::{
    form::CompactForm,
    IntoCompact as _,
    Registry,
};

/// An entire ink! project for ABI file generation purposes.
#[derive(Debug, Serialize)]
pub struct InkProject {
    registry: Registry,
    #[serde(rename = "storage")]
    layout: StorageLayout<CompactForm>,
    #[serde(rename = "contract")]
    spec: ContractSpec<CompactForm>,
}

impl InkProject {
    /// Creates a new ink! project.
    pub fn new<L, S>(layout: L, spec: S) -> Self
    where
        L: Into<StorageLayout>,
        S: Into<ContractSpec>,
    {
        let mut registry = Registry::new();
        Self {
            layout: layout.into().into_compact(&mut registry),
            spec: spec.into().into_compact(&mut registry),
            registry,
        }
    }
}
