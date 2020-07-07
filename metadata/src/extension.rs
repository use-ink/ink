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

use crate::utils::serialize_as_byte_str;
use core::{
    fmt::{
        Display,
        Formatter,
        Result as DisplayResult,
    },
    marker::PhantomData,
    str::FromStr,
};
use proc_macro2::TokenStream;
use quote::{
    quote,
    ToTokens,
};
use serde::{
    Serialize,
    Serializer,
};
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

impl ToTokens for InkProjectExtension {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let source = &self.source;
        let user = match self.user {
            Some(ref user) => quote! ( Some(#user) ),
            None => quote!(None),
        };
        let contract = &self.contract;
        quote! (
            ::ink_metadata::InkProjectExtension::new(#source, #contract, #user)
        )
        .to_tokens(tokens);
    }
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
    #[serde(serialize_with = "serialize_as_byte_str")]
    hash: [u8; 32],
    language: SourceLanguage,
    compiler: SourceCompiler,
}

impl ToTokens for InkProjectSource {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let hash = format!("{:?}", &self.hash);
        let language = &self.language;
        let compiler = &self.compiler;
        quote! (
            ::ink_metadata::InkProjectSource::new(#hash, #language, #compiler)
        )
        .to_tokens(tokens);
    }
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

impl ToTokens for SourceLanguage {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let language = &self.language;
        let version = self.version.to_string();
        quote! (
            ::ink_metadata::SourceLanguage::new(#language, #version)
        )
        .to_tokens(tokens);
    }
}

impl SourceLanguage {
    /// Constructs a new SourceLanguage.
    pub fn new(language: Language, version: Version) -> Self {
        SourceLanguage { language, version }
    }
}

impl Serialize for SourceLanguage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{} {}", self.language, self.version))
    }
}

/// Wraps [`semver::Version`] for implementing ToTokens
#[derive(Debug, Serialize)]
pub struct Version(semver::Version);

impl Default for Version {
    fn default() -> Self {
        Version(semver::Version::new(0, 0, 0))
    }
}

impl ToTokens for Version {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let version_lit = self.0.to_string();
        quote! (
            ::ink_metadata::Version::from_str(#version_lit).unwrap()
        )
        .to_tokens(tokens)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        write!(f, "{}", self.0)
    }
}

impl From<semver::Version> for Version {
    fn from(version: semver::Version) -> Self {
        Version(version)
    }
}

impl FromStr for Version {
    type Err = semver::SemVerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        semver::Version::parse(s).map(Into::into)
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

impl ToTokens for Language {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ink => quote!(::ink_metadata::Language::Ink),
            Self::Solidity => quote!(::ink_metadata::Language::Solidity),
            Self::AssemblyScript => quote!(::ink_metadata::Language::AssemblyScript),
            Self::Other(other) => quote! ( ::ink_metadata::Language::Other(#other) ),
        }
        .to_tokens(tokens)
    }
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

/// A compiler used to compile a smart contract.
#[derive(Debug)]
pub struct SourceCompiler {
    compiler: Compiler,
    version: Version,
}

impl ToTokens for SourceCompiler {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let compiler = &self.compiler;
        let version = &self.version;
        quote! (
            ::ink_metadata::SourceCompiler::new(#compiler, #version)
        )
        .to_tokens(tokens);
    }
}

impl Serialize for SourceCompiler {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{} {}", self.compiler, self.version))
    }
}

impl SourceCompiler {
    pub fn new(compiler: Compiler, version: Version) -> Self {
        SourceCompiler { compiler, version }
    }
}

/// Compilers used to compile a smart contract.
#[derive(Debug, Serialize)]
pub enum Compiler {
    RustC,
    Solang,
    Other(&'static str),
}

impl ToTokens for Compiler {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::RustC => quote!(::ink_metadata::Compiler::Solidity),
            Self::Solang => quote!(::ink_metadata::Compiler::AssemblyScript),
            Self::Other(other) => quote! ( ::ink_metadata::Compiler::Other(#other) ),
        }
        .to_tokens(tokens)
    }
}

impl Display for Compiler {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        match self {
            Self::RustC => write!(f, "rustc"),
            Self::Solang => write!(f, "solang"),
            Self::Other(other) => write!(f, "{}", other),
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

impl ToTokens for InkProjectContract {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let version = self.version.to_string();
        let authors = &self.authors;

        // initialise builder with required fields
        quote! (
            ::ink_metadata::InkProjectContract::build()
                .name(#name)
                .version(::ink_metadata::Version::parse(#version).unwrap())
                .authors(vec![
                    #( #authors, )*
                ])
        )
        .to_tokens(tokens);
        // append optional fields if present
        if let Some(ref description) = self.description {
            quote!(
                .description(#description)
            )
            .to_tokens(tokens)
        }
        if let Some(ref documentation) = self.documentation {
            let url_lit = documentation.to_string();
            quote!(
                .documentation(::ink_metadata::Url::parse(#url_lit).unwrap())
            )
            .to_tokens(tokens)
        }
        if let Some(ref repository) = self.repository {
            let url_lit = repository.to_string();
            quote!(
                .repository(::ink_metadata::Url::parse(#url_lit).unwrap())
            )
            .to_tokens(tokens)
        }
        if let Some(ref homepage) = self.homepage {
            let url_lit = homepage.to_string();
            quote!(
                .homepage(::ink_metadata::Url::parse(#url_lit).unwrap())
            )
            .to_tokens(tokens)
        }
        if let Some(ref license) = self.license {
            quote! (
                .license(#license)
            )
            .to_tokens(tokens)
        }
        // done building
        quote!( .done(); ).to_tokens(tokens)
    }
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
                version: Default::default(),
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

impl ToTokens for License {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::SpdxId(spx_id) => quote! ( ::ink_metadata::License::SpxId(#spx_id) ),
            Self::Link(url) => {
                let url_lit = url.to_string();
                quote! (
                    ::ink_metadata::License::Link(::ink_metadata::Url::parse(#url_lit).unwrap())
                )
            },
        }.to_tokens(tokens)
    }
}

/// Additional user defined metadata, can be any valid json.
#[derive(Debug, Serialize)]
pub struct InkProjectUser {
    #[serde(flatten)]
    json: serde_json::Map<String, serde_json::Value>,
}

impl ToTokens for InkProjectUser {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let json = serde_json::to_string(&self.json).expect("json should be valid json");
        quote! (
            ::ink_metadata::InkProjectUser::from_str(#json).unwrap()
        )
        .to_tokens(tokens)
    }
}

impl InkProjectUser {
    /// Constructs a new InkProjectUser
    pub fn new(json: Map<String, Value>) -> Self {
        InkProjectUser { json }
    }

    pub fn from_str(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json.as_ref()).map(Self::new)
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
#[allow(clippy::type_complexity)]
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
