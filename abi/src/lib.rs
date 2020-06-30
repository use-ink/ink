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
use serde::Serialize;

/// An entire ink! project for ABI file generation purposes.
#[derive(Debug, Serialize)]
pub struct InkProject {
    metadata_version: semver::Version,
    source: InkProjectSource,
    contract: InkProjectContract,
    user: InkProjectUser,
    spec: InkProjectSpec,
}

impl InkProject {
    /// Creates a new ink! project.
    pub fn new<M, L, S>(metadata: M, layout: L, spec: S) -> Self
    where
        M: Into<InkProjectMetadata>,
        L: Into<layout2::Layout>,
        S: Into<ContractSpec>,
    {
        let mut registry = Registry::new();
        Self {
            metadata: metadata.into(),
            layout: layout.into().into_compact(&mut registry),
            spec: spec.into().into_compact(&mut registry),
            registry,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InkProjectSource {
    hash: &'static str,
    language: &'static str,
    compiler: &'static str,
}

#[derive(Debug, Serialize)]
pub struct InkProjecContract {
    name: &'static str,
    version: semver::Version,
    authors: Vec<&'static str>,
}

impl From<InkProjectMetadataBuilder<state::Version>> for InkProjectMetadata {
    fn from(builder: InkProjectMetadataBuilder<state::Version>) -> Self {
        builder.done()
    }
}

struct InkProjectSpec {
    #[serde(flatten)] // should result in a only a "types" field
    registry: Registry,
    #[serde(rename = "storage")]
    layout: layout2::Layout<CompactForm>,
    spec: ContractSpec<CompactForm>,
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
        let version = semver::Version::parse(version.as_ref()).map_err(|_| ())?;
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
