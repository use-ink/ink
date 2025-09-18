// Copyright (C) ink! contributors.
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

//! Types for representing Solidity ABI compatibility metadata for ink! projects.

use std::borrow::Cow;

use serde::{
    Deserialize,
    Serialize,
};

/// ink! contract metadata for Solidity ABI compatible metadata generation.
#[derive(Debug, Serialize, Deserialize)]
pub struct ContractMetadata {
    /// Name of ink! contract.
    pub name: Cow<'static, str>,
    /// Metadata for all constructors of ink! contract.
    pub constructors: Vec<ConstructorMetadata>,
    /// Metadata for all messages of ink! contract.
    pub functions: Vec<FunctionMetadata>,
    /// Metadata for all events of ink! contract.
    pub events: Vec<EventMetadata>,
    /// Metadata for all errors encoded as Solidity custom errors for ink! contract.
    pub errors: Vec<ErrorMetadata>,
    /// Documentation for ink! contract.
    pub docs: Cow<'static, str>,
}

/// ink! constructor info for Solidity ABI compatible metadata generation.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConstructorMetadata {
    /// Name of ink! constructor.
    pub name: Cow<'static, str>,
    /// Parameter info for ink! constructor.
    pub inputs: Vec<ParamMetadata>,
    /// Whether the ink! constructor is marked as payable.
    pub is_payable: bool,
    /// Whether the ink! constructor is marked as default.
    pub is_default: bool,
    /// Documentation for ink! constructor.
    pub docs: Cow<'static, str>,
}

/// ink! message info for Solidity ABI compatible metadata generation.
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// Name of ink! message.
    pub name: Cow<'static, str>,
    /// Parameter info for ink! message.
    pub inputs: Vec<ParamMetadata>,
    /// Return type of ink! message.
    pub output: Option<Cow<'static, str>>,
    /// Whether the ink! message has a mutable self receiver.
    pub mutates: bool,
    /// Whether the ink! message is marked as payable.
    pub is_payable: bool,
    /// Whether the ink! message is marked as default.
    pub is_default: bool,
    /// Documentation for ink! message.
    pub docs: Cow<'static, str>,
}

/// ink! event info for Solidity ABI compatible metadata generation.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Name of ink! event.
    pub name: Cow<'static, str>,
    /// Whether the ink! event is marked as anonymous.
    pub is_anonymous: bool,
    /// Parameter info for ink! event.
    pub params: Vec<EventParamMetadata>,
    /// Documentation for ink! event.
    pub docs: Cow<'static, str>,
}

/// ink! constructor and message parameter info.
#[derive(Debug, Serialize, Deserialize)]
pub struct ParamMetadata {
    /// Name of parameter.
    pub name: Cow<'static, str>,
    /// Solidity ABI type of parameter.
    pub ty: Cow<'static, str>,
}

/// ink! event parameter info.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventParamMetadata {
    /// Name of parameter.
    pub name: Cow<'static, str>,
    /// Solidity ABI type of parameter.
    pub ty: Cow<'static, str>,
    /// Whether the parameter is marked as a topic (i.e. is indexed).
    pub is_topic: bool,
    /// Documentation for parameter.
    pub docs: Cow<'static, str>,
}

/// Error info for Solidity ABI compatible metadata generation.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMetadata {
    /// Name of error.
    pub name: Cow<'static, str>,
    /// Parameter info for error.
    pub params: Vec<ErrorParamMetadata>,
    /// Documentation for error or error variant.
    pub docs: Cow<'static, str>,
}

/// Error parameter info.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorParamMetadata {
    /// Name of parameter.
    pub name: Cow<'static, str>,
    /// Solidity ABI type of parameter.
    pub ty: Cow<'static, str>,
    /// Documentation for parameter.
    pub docs: Cow<'static, str>,
}

/// Provides [Solidity custom error metadata][abi-json] for an error type.
///
/// # Note
///
/// For enums, each variant typically corresponds to its own
/// [Solidity custom error][sol-error] type.
///
/// [abi-json]: https://docs.soliditylang.org/en/latest/abi-spec.html#json
/// [sol-error]: https://soliditylang.org/blog/2021/04/21/custom-errors/
pub trait SolErrorMetadata {
    /// Returns the metadata for the error type.
    fn error_specs() -> Vec<ErrorMetadata>;
}
