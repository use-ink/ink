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

//! ERC-20 precompile utilities for `pallet-assets`.

use crate::{
    self as ink,
    Address,
    U256,
};

pub type AssetId = u32;

/// Precompile index for trust-backed assets (pallet-assets instance 1).
pub const TRUST_BACKED: u16 = 0x0120;

/// Precompile index for foreign assets (pallet-assets instance 2).
pub const FOREIGN: u16 = 0x0121;

/// Precompile index for pool assets (pallet-assets instance 3).
pub const POOL: u16 = 0x0122;

/// Well-known ERC-20 function selectors (Keccak256 first 4 bytes).
pub mod selectors {
    pub const TOTAL_SUPPLY: [u8; 4] = [0x18, 0x16, 0x0d, 0xdd];
    pub const BALANCE_OF: [u8; 4] = [0x70, 0xa0, 0x82, 0x31];
    pub const TRANSFER: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];
    pub const ALLOWANCE: [u8; 4] = [0xdd, 0x62, 0xed, 0x3e];
    pub const APPROVE: [u8; 4] = [0x09, 0x5e, 0xa7, 0xb3];
    pub const TRANSFER_FROM: [u8; 4] = [0x23, 0xb8, 0x72, 0xdd];
}

#[ink::contract_ref(abi = "sol")]
pub trait Erc20 {
    #[ink(message)]
    fn totalSupply(&self) -> U256;

    #[ink(message)]
    fn balanceOf(&self, account: Address) -> U256;

    #[ink(message)]
    fn allowance(&self, owner: Address, spender: Address) -> U256;

    #[ink(message)]
    fn transfer(&mut self, to: Address, value: U256) -> bool;

    #[ink(message)]
    fn approve(&mut self, spender: Address, value: U256) -> bool;

    #[ink(message)]
    fn transferFrom(&mut self, from: Address, to: Address, value: U256) -> bool;
}

/// Returns the precompile address for a trust-backed asset.
#[inline]
pub fn trust_backed(asset_id: AssetId) -> Address {
    super::prefixed_address(TRUST_BACKED, asset_id)
}

/// Returns the precompile address for a foreign asset.
#[inline]
pub fn foreign(asset_id: AssetId) -> Address {
    super::prefixed_address(FOREIGN, asset_id)
}

/// Returns the precompile address for a pool asset.
#[inline]
pub fn pool(asset_id: AssetId) -> Address {
    super::prefixed_address(POOL, asset_id)
}

/// Returns the precompile address for an ERC-20 asset with a custom precompile index.
#[inline]
pub fn precompile_address(precompile_index: u16, asset_id: AssetId) -> Address {
    super::prefixed_address(precompile_index, asset_id)
}

/// Returns an `Erc20Ref` for the given precompile index and asset ID.
#[inline]
pub fn erc20(precompile_index: u16, asset_id: AssetId) -> Erc20Ref {
    precompile_address(precompile_index, asset_id).into()
}

#[cfg(all(test, feature = "xcm"))]
pub mod testing {
    use crate::env::{
        hash::{
            Blake2x128,
            HashOutput,
        },
        hash_bytes,
    };
    use scale::Encode;
    use xcm::latest::Location;

    pub fn foreign_asset_id(location: &Location) -> u32 {
        let encoded = location.encode();
        let mut hash = <Blake2x128 as HashOutput>::Type::default();
        hash_bytes::<Blake2x128>(&encoded, &mut hash);
        u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]])
    }
}
