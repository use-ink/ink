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
