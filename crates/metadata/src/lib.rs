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

#![doc(
    html_logo_url = "https://use.ink/img/crate-docs/logo.png",
    html_favicon_url = "https://use.ink/crate-docs/favicon.png"
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
extern crate core;

#[cfg(test)]
mod tests;

pub mod layout;
pub mod sol;
mod specs;
mod utils;

pub use ink_primitives::LangError;

pub use self::specs::{
    ConstructorSpec,
    ConstructorSpecBuilder,
    ContractSpec,
    ContractSpecBuilder,
    DisplayName,
    EnvironmentSpec,
    EnvironmentSpecBuilder,
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

pub use scale_info::TypeInfo;

#[cfg(feature = "derive")]
use scale_info::{
    IntoPortable as _,
    PortableRegistry,
    Registry,
    form::PortableForm,
};
use schemars::JsonSchema;
use serde::{
    Deserialize,
    Serialize,
};

/// The metadata version of the generated ink! contract.
///
/// The serialized metadata format (which this represents) is different from the
/// version of this crate or the contract for Rust semantic versioning purposes.
const METADATA_VERSION: u64 = 6;

/// An entire ink! project for metadata file generation purposes.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InkProject {
    version: u64,
    #[serde(flatten)]
    registry: PortableRegistry,
    #[serde(rename = "storage")]
    /// The layout of the storage data structure
    layout: layout::Layout<PortableForm>,
    spec: ContractSpec<PortableForm>,
}

impl InkProject {
    /// Create a new ink! project from a layout and a spec.
    pub fn new<L, S>(layout: L, spec: S) -> Self
    where
        L: Into<layout::Layout>,
        S: Into<ContractSpec>,
    {
        let mut registry = Registry::new();

        Self {
            version: METADATA_VERSION,
            layout: layout.into().into_portable(&mut registry),
            spec: spec.into().into_portable(&mut registry),
            registry: registry.into(),
        }
    }

    /// Create a new portable ink! project.
    ///
    /// The caller is responsible to register all types into the supplied registry.
    pub fn new_portable(
        layout: layout::Layout<PortableForm>,
        spec: ContractSpec<PortableForm>,
        registry: PortableRegistry,
    ) -> Self {
        Self {
            version: METADATA_VERSION,
            layout,
            spec,
            registry,
        }
    }

    /// Returns the metadata version used by the contract.
    pub fn version(&self) -> u64 {
        self.version
    }

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

/// Provides metadata about an ink! event.
///
/// Implementations must be registered into the [`static@EVENTS`] distributed slice, in
/// order to be included in the contract metadata. This is done automatically by the
/// `#[derive(ink::EventMetadata)]`
pub trait EventMetadata {
    /// The full path to the event type, usually provided by [`module_path`].
    const MODULE_PATH: &'static str;

    /// Returns the metadata of the event.
    fn event_spec() -> EventSpec;
}
