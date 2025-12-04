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

//! Precompile interfaces and utilities for ink! contracts.
//!
//! This module provides utilities for interacting with precompiles available
//! on Polkadot SDK chains with `pallet-revive`.
//!
//! # Modules
//!
//! - [`erc20`]: ERC-20 precompile utilities for `pallet-assets`
//! - [`xcm`]: XCM precompile operations (requires `xcm` feature)
//!
//! # Runtime Calls
//!
//! The [`call_runtime`] function allows contracts to execute arbitrary runtime
//! calls by specifying pallet and call indices. This is useful for calling
//! pallets that don't have dedicated precompiles.

pub mod erc20;
#[cfg(feature = "xcm")]
pub mod xcm;

pub use crate::primitives::Address;

/// Well-known precompile addresses.
pub mod addresses {
    /// ECRECOVER precompile address (0x01).
    pub const ECRECOVER: [u8; 20] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01,
    ];
    /// SHA256 precompile address (0x02).
    pub const SHA256: [u8; 20] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x02,
    ];
    /// BN128 addition precompile address (0x06).
    pub const BN128_ADD: [u8; 20] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x06,
    ];
    /// BN128 scalar multiplication precompile address (0x07).
    pub const BN128_MUL: [u8; 20] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x07,
    ];
    /// BN128 pairing precompile address (0x08).
    pub const BN128_PAIRING: [u8; 20] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x08,
    ];
    /// Utility precompile address (0x0900).
    pub const UTILITY: [u8; 20] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x09, 0x00,
    ];
    /// Storage precompile address (0x0901).
    pub const STORAGE: [u8; 20] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x09, 0x01,
    ];
    /// XCM precompile address (0x0A0000).
    pub const XCM: [u8; 20] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0A, 0, 0,
    ];
}

/// Calculates the address of a precompile at index `n`.
#[inline]
pub fn fixed_address(n: u16) -> Address {
    let shifted = (n as u32) << 16;
    let suffix = shifted.to_be_bytes();
    let mut address = [0u8; 20];
    let mut i = 16;
    while i < address.len() {
        address[i] = suffix[i - 16];
        i += 1;
    }
    Address::from(address)
}

/// Calculates the address of a precompile at index `n` with an additional prefix.
#[inline]
pub fn prefixed_address(n: u16, prefix: u32) -> Address {
    let address = fixed_address(n);
    let mut address_bytes: [u8; 20] = address.into();
    address_bytes[..4].copy_from_slice(&prefix.to_be_bytes());
    Address::from(address_bytes)
}

// =============================================================================
// Runtime Call API (requires xcm feature)
// =============================================================================

#[cfg(feature = "xcm")]
use crate::{
    env,
    prelude::{
        string::String,
        vec,
        vec::Vec,
    },
};
#[cfg(feature = "xcm")]
use ::xcm::DoubleEncoded;
#[cfg(feature = "xcm")]
pub use ::xcm::prelude::*;
#[cfg(feature = "xcm")]
use scale::Encode;

/// Errors that can occur during runtime call execution.
#[cfg(feature = "xcm")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchError {
    /// Failed to weigh the XCM message.
    WeighFailed,
    /// Failed to execute the XCM message.
    ExecuteFailed {
        /// Raw revert bytes returned by the XCM precompile, if any.
        raw: Vec<u8>,
    },
}

#[cfg(feature = "xcm")]
impl DispatchError {
    /// Returns the raw revert bytes returned by the XCM precompile, if any.
    pub fn raw(&self) -> Option<&[u8]> {
        match self {
            DispatchError::WeighFailed => None,
            DispatchError::ExecuteFailed { raw } => Some(raw.as_slice()),
        }
    }

    /// Tries to decode the revert reason string emitted by the XCM precompile.
    ///
    /// The XCM precompile reverts with an ABI encoded string. This helper decodes it
    /// into a human-readable message when available.
    pub fn revert_reason(&self) -> Option<String> {
        let raw = self.raw()?;
        decode_revert_message(raw)
    }
}

#[cfg(feature = "xcm")]
impl core::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DispatchError::WeighFailed => write!(f, "failed to weigh XCM message"),
            DispatchError::ExecuteFailed { .. } => {
                write!(f, "failed to execute XCM message")
            }
        }
    }
}

/// Execute a runtime call by pallet index and call index.
#[cfg(feature = "xcm")]
pub fn call_runtime<Args>(
    pallet_index: u8,
    call_index: u8,
    args: Args,
) -> Result<(), DispatchError>
where
    Args: Encode,
{
    let mut encoded_call = vec![pallet_index, call_index];
    args.encode_to(&mut encoded_call);

    let call: DoubleEncoded<()> = encoded_call.into();
    let xcm_msg: Xcm<()> = Xcm::builder_unsafe()
        .transact(
            OriginKind::SovereignAccount,
            Weight::from_parts(u64::MAX, u64::MAX),
            call,
        )
        .build();

    let versioned_msg = VersionedXcm::from(xcm_msg);
    let weight =
        env::xcm_weigh(&versioned_msg).map_err(|_| DispatchError::WeighFailed)?;
    let execution = env::xcm_execute(&versioned_msg, weight)
        .map_err(|_| DispatchError::ExecuteFailed { raw: Vec::new() })?;
    if execution.did_revert {
        return Err(DispatchError::ExecuteFailed {
            raw: execution.data,
        });
    }
    Ok(())
}

#[cfg(feature = "xcm")]
fn decode_revert_message(raw: &[u8]) -> Option<String> {
    const REVERT_SELECTOR: [u8; 4] = [0x08, 0xc3, 0x79, 0xa0];

    if raw.len() < 4 + 32 + 32 {
        return None;
    }
    if raw.get(..4)? != REVERT_SELECTOR {
        return None;
    }

    let mut offset_bytes = [0u8; 4];
    offset_bytes.copy_from_slice(raw.get(4..8)?);
    let offset = u32::from_be_bytes(offset_bytes);
    if offset != 32 {
        return None;
    }

    let len_word_start = 4 + 32;
    let len_word_end = len_word_start + 32;
    let mut len_bytes = [0u8; 4];
    len_bytes.copy_from_slice(raw.get((len_word_end - 4)..len_word_end)?);
    let len = u32::from_be_bytes(len_bytes) as usize;

    let data_start = len_word_end;
    let data_end = data_start.checked_add(len)?;
    if raw.len() < data_end {
        return None;
    }
    let reason = &raw[data_start..data_end];
    Some(String::from_utf8_lossy(reason).into_owned())
}
