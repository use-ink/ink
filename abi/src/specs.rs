// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    AbiType,
    TupleVec,
};
use core::marker::PhantomData;
use serde::{
    Deserialize,
    Serialize,
};

/// Describes a contract.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ContractSpec<DeployParams, Messages, Events>
where
    DeployParams: TupleVec, // <Item = ParamSpec<T>>
    Messages: TupleVec,     // <Item = MessageSpec<T>>
{
    /// The name of the contract.
    name: &'static str,
    /// The deploy handler of the contract.
    deploy: DeploySpec<DeployParams>,
    /// The external messages of the contract.
    messages: Messages,
    /// The events of the contract.
    events: Events,
    /// The contract documentation.
    documentation: Vec<&'static str>,
}

/// Describes the deploy handler of a contract.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct DeploySpec<Params>
where
    Params: TupleVec, // <Item = ParamSpec<T>>
{
    /// The parameters of the deploy handler.
    args: Params,
    /// The deploy handler documentation.
    documentation: Vec<&'static str>,
}

/// Describes a contract message.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct MessageSpec<Params, RetType>
where
    Params: TupleVec, // <Item = ParamSpec<T>>
    RetType: AbiType,
    TypeSpec<RetType>: Serialize,
{
    /// The name of the message.
    name: &'static str,
    /// The selector hash of the message.
    selector: u64,
    /// If the message is allowed to mutate the contract state.
    mutates: bool,
    /// The parameters of the message.
    args: Params,
    /// The return type of the message.
    return_type: ReturnTypeSpec<RetType>,
    /// The message documentation.
    documentation: Vec<&'static str>,
}

/// Describes an event definition.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct EventSpec<Params>
where
    Params: TupleVec, // <Item = EventParamSpec<T>>
{
    /// The name of the event.
    name: &'static str,
    /// The event arguments.
    args: Params,
    /// The event documentation.
    documentation: Vec<&'static str>,
}

/// Describes a pair of parameter name and type.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound(serialize = "TypeSpec<T>: Serialize,"))]
pub struct EventParamSpec<T>
where
    T: AbiType,
{
    /// The name of the parameter.
    name: &'static str,
    /// If the event parameter is indexed.
    indexed: bool,
    /// The type of the parameter.
    #[serde(rename = "type")]
    ty: TypeSpec<T>,
}

/// Describes the return type of a contract message.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ReturnTypeSpec<T>
where
    T: AbiType,
    TypeSpec<T>: Serialize,
{
    #[serde(rename = "type")]
    opt_type: Option<TypeSpec<T>>,
}

/// Describes a pair of parameter name and type.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound(serialize = "TypeSpec<T>: Serialize,"))]
pub struct ParamSpec<T>
where
    T: AbiType,
    // TypeSpec<T>: Serialize,
{
    /// The name of the parameter.
    name: &'static str,
    /// The type of the parameter.
    #[serde(rename = "type")]
    ty: TypeSpec<T>,
}

/// Describes a type.
#[derive(Debug, PartialEq, Eq)]
pub struct TypeSpec<T>
where
    T: AbiType,
{
    /// Marker used so that we do not need an instance of the specified type.
    marker: PhantomData<fn() -> T>,
}

impl<T> TypeSpec<T>
where
    T: AbiType,
{
    /// Creates a new type spec for the given type.
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

/// Describes a custom type definition and all of its fields and subfields.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct CustomTypeSpec<Fields>
where
    Fields: TupleVec, // <Item = CustomTypeFieldSpec<T>>
{
    /// The name of the custom type.
    name: &'static str,
    /// The fields of the custom type.
    fields: Fields,
}

/// Describes a field of a custom type definition.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct CustomTypeFieldSpec<T>
where
    T: AbiType,
{
    /// The name of the field.
    name: &'static str,
    /// The type of the field.
    #[serde(rename = "type")]
    ty: T,
}

/// Describes the layout of the storage.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StorageLayout<StorageFields>
where
    StorageFields: TupleVec, // <Item = StorageField<T>>
{
    /// The fields of the storage layout.
    fields: StorageFields,
}

/// Describes a field or sub-field of the layout of the storage.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StorageField<T>
where
    T: AbiType,
{
    /// The name of the storage field or sub-field.
    name: &'static str,
    /// The type of the storage field or sub-field.
    ty: T,
    /// The key bounds for the storage field or sub-field.
    key: KeyBounds,
}

/// A key.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Key(pub [u8; 32]);

/// The key bounds of a storage field.
///
/// This defines in which bounds a storage field might have stored values.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct KeyBounds {
    /// The key offset.
    key: Key,
    /// The length of all contiguous keys starting at the key offset.
    len: usize,
}
