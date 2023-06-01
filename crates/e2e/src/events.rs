// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use ink::codegen::ContractCallBuilder;
use ink_env::{
    call::{
        utils::{
            ReturnType,
            Set,
        },
        Call,
        ExecutionInput,
        FromAccountId,
    },
    Environment,
};
use ink_primitives::MessageResult;
use pallet_contracts_primitives::ExecReturnValue;
use sp_core::Pair;
#[cfg(feature = "std")]
use std::{
    collections::BTreeMap,
    fmt::Debug,
    marker::PhantomData,
    path::PathBuf,
};

use subxt::{
    blocks::ExtrinsicEvents,
    config::ExtrinsicParams,
    events::EventDetails,
    ext::{
        scale_decode,
        scale_encode,
        scale_value::{
            Composite,
            Value,
            ValueDef,
        },
    },
    tx::PairSigner,
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

impl<E> subxt::events::StaticEvent for ContractInstantiatedEvent<E>
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

impl<E> subxt::events::StaticEvent for CodeStoredEvent<E>
where
    E: Environment,
{
    const PALLET: &'static str = "Contracts";
    const EVENT: &'static str = "CodeStored";
}
