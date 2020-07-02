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

use core::marker::PhantomData;
use semver::Version;
use serde::Serialize;
use serde_json::{Map, Value};
use url::Url;

/// Additional metadata supplied externally, e.g. by `cargo-contract`.
#[derive(Debug, Serialize)]
pub struct InkProjectExtension {
    source: InkProjectSource,
    contract: InkProjectContract,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<InkProjectUser>,
}

impl InkProjectExtension {
    /// Constructs a new InkProjectExtension.
    pub fn new(source: InkProjectSource, contract: InkProjectContract, user: Option<InkProjectUser>) -> Self {
        InkProjectExtension { source, contract, user }
    }
}

#[derive(Debug, Serialize)]
pub struct InkProjectSource {
    hash: [u8; 32],
    language: SourceLanguage,
    compiler: SourceCompiler,
}

impl InkProjectSource {
    /// Constructs a new InkProjectSource.
    pub fn new(hash: [u8; 32], language: SourceLanguage, compiler: SourceCompiler) -> Self {
        InkProjectSource { hash, language, compiler }
    }
}

/// The language and version in which a smart contract is written.
// todo: custom serialize e.g. `ink! v0.3.0`
#[derive(Debug, Serialize)]
pub struct SourceLanguage {
    language: Language,
    version: Version,
}

impl SourceLanguage {
    /// Constructs a new SourceLanguage.
    pub fn new(language: Language, version: Version) -> Self {
        SourceLanguage { language, version }
    }
}

/// The language in which the smart contract is written.
#[derive(Debug, Serialize)]
pub enum Language {
    Ink,
    Solidity,
    AssemblyScript,
    Other(&'static str),
}

/// The compilers used to compile a smart contract.
// todo: custom serialize e.g. `ink! v0.3.0 (rustc 1.41.0)`
#[derive(Debug, Serialize)]
pub struct SourceCompiler {
    high_level: CompilerInfo,
    low_level: CompilerInfo,
}

impl SourceCompiler {
    /// Constructs a new SourceCompiler.
    pub fn new(high_level: CompilerInfo, low_level: CompilerInfo) -> Self {
        SourceCompiler { high_level, low_level }
    }
}

/// A compiler used to compile a smart contract.
#[derive(Debug, Serialize)]
pub struct CompilerInfo {
    compiler: Compiler,
    version: Version,
}

/// Compilers used to compile a smart contract.
#[derive(Debug, Serialize)]
pub enum Compiler {
    Ink,
    RustC,
    Solang,
    LLVM,
}

/// Metadata about a smart contract.
#[derive(Debug, Serialize)]
pub struct InkProjectContract {
    name: String,
    version: Version,
    authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    documentation: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<License>,
}

impl InkProjectContract {
    /// Constructs a new InkProjectContractBuilder.
    pub fn build() -> InkProjectContractBuilder<Missing<state::Name>, Missing<state::Version>, Missing<state::Authors>> {
        InkProjectContractBuilder {
            contract: Self {
                name: Default::default(),
                version: Version::new(0, 0, 0),
                authors: vec![],
                description: None,
                documentation: None,
                repository: None,
                homepage: None,
                license: None
            },
            marker: Default::default()
        }
    }
}

/// The license of a smart contract
#[derive(Debug, Serialize)]
pub enum License {
    /// An [SPDX identifier](https://spdx.org/licenses/)
    SpdxId(String),
    /// A URL to a custom license
    Link(Url),
}

/// Additional user defined metadata, can be any valid json.
#[derive(Debug, Serialize)]
pub struct InkProjectUser {
    #[serde(flatten)]
    json: serde_json::Map<String, serde_json::Value>
}

impl InkProjectUser {
    /// Constructs a new InkProjectUser
    pub fn new(json: Map<String, Value>) -> Self {
        InkProjectUser { json }
    }
}

/// Type state for builders to tell that some mandatory state has not yet been set
/// yet or to fail upon setting the same state multiple times.
pub struct Missing<S>(PhantomData<fn() -> S>);

mod state {
    //! Type states that tell what state of the project metadata has not
    //! yet been set properly for a valid construction.

    /// Type state for the name of the project.
    pub struct Name;

    /// Type state for the version of the project.
    pub struct Version;

    /// Type state for the authors of the project.
    pub struct Authors;
}

pub struct InkProjectContractBuilder<Name, Version, Authors> {
    contract: InkProjectContract,
    marker: PhantomData<fn() -> (Name, Version, Authors)>
}

impl<V, A> InkProjectContractBuilder<Missing<state::Name>, V, A> {
    pub fn name<S>(self, name: S) -> InkProjectContractBuilder<state::Name, V, A>
    where
        S: AsRef<str>
    {
        InkProjectContractBuilder {
            contract: InkProjectContract {
                name: name.as_ref().to_owned(),
                ..self.contract
            },
            marker: PhantomData,
        }
    }
}

impl<N, A> InkProjectContractBuilder<N, Missing<state::Version>, A> {
    pub fn version(self, version: Version) -> InkProjectContractBuilder<N, state::Version, A> {
        InkProjectContractBuilder {
            contract: InkProjectContract {
                version,
                ..self.contract
            },
            marker: PhantomData,
        }
    }
}

impl<N, V> InkProjectContractBuilder<N, V, Missing<state::Authors>> {
    pub fn authors<I, S>(self, authors: I) -> InkProjectContractBuilder<N, V, state::Authors>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>
    {
        InkProjectContractBuilder {
            contract: InkProjectContract {
                authors: authors.into_iter().map(|s| s.as_ref().into()).collect(),
                ..self.contract
            },
            marker: PhantomData,
        }
    }
}

impl InkProjectContractBuilder<state::Name, state::Version, state::Authors> {
    fn done(self) -> InkProjectContract {
        self.contract
    }
}
