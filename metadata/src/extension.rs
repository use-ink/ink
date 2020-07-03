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

use core::fmt::{Display, Formatter, Result as DisplayResult};
use core::marker::PhantomData;
use semver::Version;
use serde::{Serialize, Serializer};
use serde_json::{
    Map,
    Value,
};
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
    pub fn new(
        source: InkProjectSource,
        contract: InkProjectContract,
        user: Option<InkProjectUser>,
    ) -> Self {
        InkProjectExtension {
            source,
            contract,
            user,
        }
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
    pub fn new(
        hash: [u8; 32],
        language: SourceLanguage,
        compiler: SourceCompiler,
    ) -> Self {
        InkProjectSource {
            hash,
            language,
            compiler,
        }
    }
}

/// The language and version in which a smart contract is written.
#[derive(Debug)]
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

impl Serialize for SourceLanguage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&format!("{} {}", self.language, self.version))
    }
}

/// The language in which the smart contract is written.
#[derive(Debug)]
pub enum Language {
    Ink,
    Solidity,
    AssemblyScript,
    Other(&'static str),
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        match self {
            Self::Ink => write!(f, "ink!"),
            Self::Solidity => write!(f, "Solidity"),
            Self::AssemblyScript => write!(f, "AssemblyScript"),
            Self::Other(lang) => write!(f, "{}", lang),
        }
    }
}

/// The compilers used to compile a smart contract.
#[derive(Debug)]
pub struct SourceCompiler {
    high_level: CompilerInfo,
    low_level: CompilerInfo,
}

impl Serialize for SourceCompiler {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&format!("{} ({})", self.high_level, self.low_level))
    }
}

impl SourceCompiler {
    /// Constructs a new SourceCompiler.
    pub fn new(high_level: CompilerInfo, low_level: CompilerInfo) -> Self {
        SourceCompiler {
            high_level,
            low_level,
        }
    }
}

/// A compiler used to compile a smart contract.
#[derive(Debug)]
pub struct CompilerInfo {
    compiler: Compiler,
    version: Version,
}

impl Display for CompilerInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        write!(f, "{} {}", self.compiler, self.version)
    }
}

impl CompilerInfo {
    pub fn new(compiler: Compiler, version: Version) -> Self {
        CompilerInfo { compiler, version }
    }
}

/// Compilers used to compile a smart contract.
#[derive(Debug, Serialize)]
pub enum Compiler {
    Ink,
    RustC,
    Solang,
    LLVM,
}

impl Display for Compiler {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        match self {
            Self::Ink => write!(f, "ink!"),
            Self::RustC => write!(f, "rustc"),
            Self::Solang => write!(f, "solang"),
            Self::LLVM => write!(f, "llvm"),
        }
    }
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
    pub fn build() -> InkProjectContractBuilder<
        Missing<state::Name>,
        Missing<state::Version>,
        Missing<state::Authors>,
    > {
        InkProjectContractBuilder {
            contract: Self {
                name: Default::default(),
                version: Version::new(0, 0, 0),
                authors: vec![],
                description: None,
                documentation: None,
                repository: None,
                homepage: None,
                license: None,
            },
            marker: Default::default(),
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
    json: serde_json::Map<String, serde_json::Value>,
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

/// Build an [`InkProjectContract`], ensuring required fields are supplied
///
/// # Example
///
/// ```
/// # use crate::ink_metadata::InkProjectContract;
/// # use semver::Version;
/// # use url::Url;
/// // contract metadata with the minimum set of required fields
/// let metadata1: InkProjectContract =
///     InkProjectContract::build()
///         .name("example")
///         .version(Version::new(0, 1, 0))
///         .authors(vec!["author@example.com"])
///         .done();
///
/// // contract metadata with optional fields
/// let metadata2: InkProjectContract =
///     InkProjectContract::build()
///         .name("example")
///         .version(Version::new(0, 1, 0))
///         .authors(vec!["author@example.com"])
///         .description("description")
///         .documentation(Url::parse("http://example.com").unwrap())
///         .repository(Url::parse("http://example.com").unwrap())
///         .homepage(Url::parse("http://example.com").unwrap())
///         .done();
/// ```
pub struct InkProjectContractBuilder<Name, Version, Authors> {
    contract: InkProjectContract,
    marker: PhantomData<fn() -> (Name, Version, Authors)>,
}

impl<V, A> InkProjectContractBuilder<Missing<state::Name>, V, A> {
    /// Set the contract name (required)
    pub fn name<S>(self, name: S) -> InkProjectContractBuilder<state::Name, V, A>
    where
        S: AsRef<str>,
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
    /// Set the contract version (required)
    pub fn version(
        self,
        version: Version,
    ) -> InkProjectContractBuilder<N, state::Version, A> {
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
    /// Set the contract authors (required)
    pub fn authors<I, S>(
        self,
        authors: I,
    ) -> InkProjectContractBuilder<N, V, state::Authors>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
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

impl<N, V, A> InkProjectContractBuilder<N, V, A> {
    /// Set the contract description (optional)
    pub fn description<S>(mut self, description: S) -> Self
    where
        S: AsRef<str>,
    {
        self.contract.description = Some(description.as_ref().to_owned());
        self
    }

    /// Set the contract documentation url (optional)
    pub fn documentation(mut self, documentation: Url) -> Self {
        self.contract.documentation = Some(documentation);
        self
    }

    /// Set the contract documentation url (optional)
    pub fn repository(mut self, repository: Url) -> Self {
        self.contract.repository = Some(repository);
        self
    }

    /// Set the contract homepage url (optional)
    pub fn homepage(mut self, homepage: Url) -> Self {
        self.contract.homepage = Some(homepage);
        self
    }

    /// Set the contract license (optional)
    pub fn license(mut self, license: License) -> Self {
        self.contract.license = Some(license);
        self
    }
}

impl InkProjectContractBuilder<state::Name, state::Version, state::Authors> {
    pub fn done(self) -> InkProjectContract {
        self.contract
    }
}
