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
    form::PortableForm,
    IntoPortable as _,
    PortableRegistry,
    Registry,
};
use serde::{
    Deserialize,
    Serialize,
};

/// Versioned ink! project metadata.
///
/// # Note
///
/// Represents the version of the serialized metadata *format*, which is distinct from the version
/// of this crate for Rust semver compatibility.
#[derive(Debug, Serialize, Deserialize)]
pub enum MetadataVersioned {
    /// Version 0 placeholder. Represents the original non-versioned metadata format.
    V0(MetadataVersionDeprecated),
    /// Version 1 of the contract metadata.
    V1(InkProject),
}

impl From<InkProject> for MetadataVersioned {
    fn from(ink_project: InkProject) -> Self {
        MetadataVersioned::V1(ink_project)
    }
}

/// Enum to represent a deprecated metadata version that cannot be instantiated.
#[derive(Debug, Serialize, Deserialize)]
pub enum MetadataVersionDeprecated {}

/// An entire ink! project for metadata file generation purposes.
#[derive(Debug, Serialize, Deserialize)]
pub struct InkProject {
    #[serde(flatten)]
    registry: PortableRegistry,
    #[serde(rename = "storage")]
    /// The layout of the storage data structure
    layout: layout::Layout<PortableForm>,
    spec: ContractSpec<PortableForm>,
}

impl InkProject {
    pub fn new<L, S>(layout: L, spec: S) -> Self
    where
        L: Into<layout::Layout>,
        S: Into<ContractSpec>,
    {
        let mut registry = Registry::new();

        Self {
            layout: layout.into().into_portable(&mut registry),
            spec: spec.into().into_portable(&mut registry),
            registry: registry.into(),
        }
    }
}

impl InkProject {
    /// Returns a read-only registry of types in the contract.
    pub fn registry(&self) -> &PortableRegistry {
        &self.registry
    }

    /// Returns the storage layout of the contract.
    pub fn layout(&self) -> &layout::Layout<PortableForm> {
        &self.layout
    }

    /// Returns the specification of the contract.
    pub fn spec(&self) -> &ContractSpec<PortableForm> {
        &self.spec
    }
}
