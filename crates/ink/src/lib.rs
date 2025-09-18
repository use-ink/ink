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

#[macro_use]
#[doc(hidden)]
pub mod option_info;

#[macro_use]
#[doc(hidden)]
pub mod result_info;

#[cfg_attr(not(feature = "show-codegen-docs"), doc(hidden))]
pub mod codegen;

pub use ink_env::reflect;

mod contract_ref;
mod env_access;
mod message_builder;
pub mod sol;

pub use ink_env as env;
#[cfg(feature = "std")]
pub use ink_metadata as metadata;
pub use ink_prelude as prelude;
pub use ink_primitives as primitives;
pub use ink_primitives::abi;
pub use scale;
#[cfg(feature = "std")]
pub use scale_info;
#[cfg(feature = "xcm")]
pub use xcm;

pub extern crate polkavm_derive;
#[doc(hidden)]
pub use polkavm_derive::*;

pub mod storage {
    pub mod traits {
        pub use ink_macro::{
            Storable,
            StorableHint,
            StorageKey,
            StorageLayout,
        };
        pub use ink_storage::traits::*;
    }
    pub use ink_storage::{
        Lazy,
        Mapping,
        StorageVec,
    };
}

pub use self::{
    contract_ref::ToAddr,
    env_access::EnvAccess,
    prelude::IIP2_WILDCARD_COMPLEMENT_SELECTOR,
};
pub use ink_macro::{
    Event,
    EventMetadata,
    SolDecode,
    SolEncode,
    SolErrorDecode,
    SolErrorEncode,
    SolErrorMetadata,
    blake2x256,
    contract,
    error,
    event,
    scale_derive,
    selector_bytes,
    selector_id,
    storage_item,
    test,
    trait_definition,
};
pub use ink_primitives::{
    Address,
    ConstructorResult,
    H160,
    H256,
    LangError,
    MessageResult,
    SolDecode,
    SolEncode,
    U256,
};

#[cfg(feature = "std")]
#[doc(hidden)]
pub use linkme;

#[cfg(feature = "std")]
use ink_metadata::EventSpec;

/// Any event which derives `#[derive(ink::EventMetadata)]` and is used in the contract
/// binary will have its implementation added to this distributed slice at linking time.
#[cfg(feature = "std")]
#[linkme::distributed_slice]
#[linkme(crate = linkme)]
pub static CONTRACT_EVENTS: [fn() -> EventSpec] = [..];

/// Collect the [`EventSpec`] metadata of all event definitions linked and used in the
/// binary.
#[cfg(feature = "std")]
pub fn collect_events() -> Vec<EventSpec> {
    CONTRACT_EVENTS.iter().map(|event| event()).collect()
}

/// Any event whose parameters type implement `ink::SolDecode` and `ink::SolEncode`
/// and is used in the contract binary will have its implementation added to this
/// distributed slice at linking time.
#[cfg(all(feature = "std", any(ink_abi = "sol", ink_abi = "all")))]
#[linkme::distributed_slice]
#[linkme(crate = linkme)]
pub static CONTRACT_EVENTS_SOL: [fn() -> ink_metadata::sol::EventMetadata] = [..];

/// Collect the Solidity ABI compatible metadata of all event definitions linked and used
/// in the binary.
#[cfg(all(feature = "std", any(ink_abi = "sol", ink_abi = "all")))]
pub fn collect_events_sol() -> Vec<ink_metadata::sol::EventMetadata> {
    crate::CONTRACT_EVENTS_SOL
        .iter()
        .map(|event| event())
        .collect()
}

/// Any error which derives `#[derive(ink::SolErrorMetadata)]` and is used in the contract
/// binary will have its implementation added to this distributed slice at linking time.
#[cfg(feature = "std")]
#[linkme::distributed_slice]
#[linkme(crate = linkme)]
pub static CONTRACT_ERRORS_SOL: [fn() -> Vec<ink_metadata::sol::ErrorMetadata>] = [..];

/// Collect the Solidity ABI compatible metadata of all error definitions encoded as
/// Solidity custom errors that are linked and used in the binary.
#[cfg(feature = "std")]
pub fn collect_errors_sol() -> Vec<ink_metadata::sol::ErrorMetadata> {
    crate::CONTRACT_ERRORS_SOL
        .iter()
        .flat_map(|event| event())
        .collect()
}
