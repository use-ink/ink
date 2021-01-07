// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

#[cfg(test)]
mod tests;

pub mod layout;
mod specs;
mod utils;

pub use self::specs::{
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
    Selector,
    TypeSpec,
};

use impl_serde::serialize as serde_hex;

#[cfg(feature = "derive")]
use scale_info::{
    form::CompactForm,
    IntoCompact as _,
    Registry,
    RegistryReadOnly,
};
use serde::{
    Deserialize,
    Serialize,
};

/// An entire ink! project for metadata file generation purposes.
#[derive(Debug, Serialize, Deserialize)]
pub struct InkProject {
    #[serde(flatten)]
    registry: RegistryReadOnly,
    #[serde(rename = "storage")]
    /// The layout of the storage data structure
    layout: layout::Layout<CompactForm>,
    spec: ContractSpec<CompactForm>,
}

impl InkProject {
    pub fn new<L, S>(layout: L, spec: S) -> Self
    where
        L: Into<layout::Layout>,
        S: Into<ContractSpec>,
    {
        let mut registry = Registry::new();

        Self {
            layout: layout.into().into_compact(&mut registry),
            spec: spec.into().into_compact(&mut registry),
            registry: registry.into(),
        }
    }
}

impl InkProject {
    /// Returns a read-only registry of types in the contract.
    pub fn registry(&self) -> &RegistryReadOnly {
        &self.registry
    }

    /// Returns the storage layout of the contract.
    pub fn layout(&self) -> &layout::Layout<CompactForm> {
        &self.layout
    }

    /// Returns the specification of the contract.
    pub fn spec(&self) -> &ContractSpec<CompactForm> {
        &self.spec
    }
}
