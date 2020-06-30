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

#[cfg(test)]
mod tests;

pub mod layout2;
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
    TypeSpec,
};
use core::marker::PhantomData;
#[cfg(feature = "derive")]
use scale_info::{
    form::CompactForm,
    IntoCompact as _,
    Registry,
};
use semver::Version;
use serde::Serialize;
use url::Url;

const METADATA_VERSION: &str = "0.1.0";

/// An entire ink! project for ABI file generation purposes.
#[derive(Debug, Serialize)]
pub struct InkProject {
    metadata_version: Version,
    #[serde(flatten)]
    extension: InkProjectExtension,
    spec: InkProjectSpec,
}

impl InkProject {
    pub fn new(extension: InkProjectExtension, spec: InkProjectSpec) -> Self {
        let metadata_version= Version::parse(METADATA_VERSION).expect("METADATA_VERSION is a valid semver string");
        InkProject {
            metadata_version,
            extension,
            spec
        }
    }
}

/// Additional metadata supplied externally, e.g. by a tool such as `cargo-contract`
#[derive(Debug, Serialize)]
pub struct InkProjectExtension {
    source: InkProjectSource,
    contract: InkProjectContract,
    user: InkProjectUser,
}

#[derive(Debug, Serialize)]
pub struct InkProjectSource {
    hash: &'static str,
    language: &'static str,
    compiler: &'static str,
}

#[derive(Debug, Serialize)]
pub struct InkProjectContract {
    name: &'static str,
    version: Version,
    authors: Vec<&'static str>,
    description: &'static str,
    documentation: Url,
    repository: Url,
    homepage: Url,
    license: &'static str,
}

#[derive(Debug, Serialize)]
pub struct InkProjectUser {
    #[serde(flatten)]
    json: serde_json::Map<String, serde_json::Value>
}

impl From<InkProjectMetadataBuilder<state::Version>> for InkProjectMetadata {
    fn from(builder: InkProjectMetadataBuilder<state::Version>) -> Self {
        builder.done()
    }
}

#[derive(Debug, Serialize)]
struct InkProjectSpec {
    #[serde(flatten)] // should result in a only a "types" field
    registry: Registry,
    #[serde(rename = "storage")]
    layout: layout2::Layout<CompactForm>,
    spec: ContractSpec<CompactForm>,
}

impl InkProjectSpec {
    /// Creates a new ink! project.
    pub fn new<M, L, S>(layout: L, spec: S) -> Self
    where
        L: Into<layout2::Layout>,
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

/// Type state for builders to tell that some mandatory state has not yet been set
/// yet or to fail upon setting the same state multiple times.
pub struct Missing<S>(PhantomData<fn() -> S>);

mod state {
    //! Type states that tell what state of the project metadata has not
    //! yet been set properly for a valid construction.

    /// Type state for the version of the project metadata.
    pub struct Version;
}

pub struct InkProjectMetadataBuilder<Version> {
    metadata: InkProjectMetadata,
    marker: PhantomData<fn() -> (Version)>
}

impl InkProjectMetadataBuilder<Missing<state::Version>> {
    // todo: error type?
    pub fn version<S>(self, version: S) -> Result<InkProjectMetadataBuilder<state::Version>, ()> {
        let version = Version::parse(version.as_ref()).map_err(|_| ())?;
        Ok(InkProjectMetadataBuilder {
            metadata: InkProjectMetadata {
                version,
                ..self.metadata
            },
            marker: PhantomData,
        })
    }
}

impl InkProjectMetadataBuilder<state::Version> {
    fn done(self) -> InkProjectMetadata {
        self.metadata
    }
}
