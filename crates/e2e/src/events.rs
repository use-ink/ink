// Copyright (C) Parity Technologies (UK) Ltd.
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

use ink_env::Environment;
#[cfg(feature = "std")]
use std::fmt::Debug;

use subxt::{
    events::StaticEvent,
    ext::{
        scale_decode,
        scale_encode,
    },
};

/// A contract was successfully instantiated.
#[derive(
    Debug,
    scale::Decode,
    scale::Encode,
    scale_decode::DecodeAsType,
    scale_encode::EncodeAsType,
)]
#[decode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_decode")]
#[encode_as_type(crate_path = "subxt::ext::scale_encode")]
pub struct ContractInstantiatedEvent<E: Environment> {
    /// Account id of the deployer.
    pub deployer: E::AccountId,
    /// Account id where the contract was instantiated to.
    pub contract: E::AccountId,
}

impl<E> StaticEvent for ContractInstantiatedEvent<E>
where
    E: Environment,
{
    const PALLET: &'static str = "Contracts";
    const EVENT: &'static str = "Instantiated";
}

/// Code with the specified hash has been stored.
#[derive(
    Debug,
    scale::Decode,
    scale::Encode,
    scale_decode::DecodeAsType,
    scale_encode::EncodeAsType,
)]
#[decode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_decode")]
#[encode_as_type(crate_path = "subxt::ext::scale_encode")]
pub struct CodeStoredEvent<E: Environment> {
    /// Hash under which the contract code was stored.
    pub code_hash: E::Hash,
}

impl<E> StaticEvent for CodeStoredEvent<E>
where
    E: Environment,
{
    const PALLET: &'static str = "Contracts";
    const EVENT: &'static str = "CodeStored";
}

#[derive(
    scale::Decode,
    scale::Encode,
    scale_decode::DecodeAsType,
    scale_encode::EncodeAsType,
    Debug,
)]
#[decode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_decode")]
#[encode_as_type(crate_path = "subxt::ext::scale_encode")]
/// A custom event emitted by the contract.
pub struct ContractEmitted<E: Environment> {
    pub contract: E::AccountId,
    pub data: Vec<u8>,
}

impl<E> StaticEvent for ContractEmitted<E>
where
    E: Environment,
{
    const PALLET: &'static str = "Contracts";
    const EVENT: &'static str = "ContractEmitted";
}

/// A decoded event with its associated topics.
pub struct EventWithTopics<T> {
    pub topics: Vec<sp_core::H256>,
    pub event: T,
}
