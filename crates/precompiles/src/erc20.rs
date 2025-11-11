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

//! ERC-20 precompile interface for `pallet-assets`.
//!
//! This module provides the standard ERC-20 token interface for interacting with
//! assets managed by `pallet-assets`.
//!
//! # References
//!
//! - [Polkadot SDK Assets Precompile](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/assets/precompiles/src/lib.rs)
//! - [Polkadot SDK Assets Precompile Solidity Interface](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/primitives/ethereum-standards/src/IERC20.sol)
//! - [ERC-20 Token Standard](https://eips.ethereum.org/EIPS/eip-20)

use ink::{
    Address,
    U256,
};

/// Type alias for asset IDs.
pub type AssetId = u32;

/// Defines the ERC-20 interface of the Asset Hub precompile.
#[ink::contract_ref(abi = "sol")]
pub trait Erc20 {
    /// Returns the total supply of tokens.
    ///
    /// # Solidity Signature
    ///
    /// ```solidity
    /// function totalSupply() external view returns (uint256);
    /// ```
    #[ink(message)]
    #[allow(non_snake_case)]
    fn totalSupply(&self) -> U256;

    /// Returns the balance of an account.
    ///
    /// # Arguments
    /// * `account` - The address to query the balance of
    ///
    /// # Solidity Signature
    ///
    /// ```solidity
    /// function balanceOf(address account) external view returns (uint256);
    /// ```
    #[ink(message)]
    #[allow(non_snake_case)]
    fn balanceOf(&self, account: Address) -> U256;

    /// Transfers tokens to another account.
    ///
    /// # Arguments
    /// * `to` - The recipient address
    /// * `value` - The amount of tokens to transfer
    ///
    /// # Returns
    ///
    /// Returns `true` if the transfer was successful.
    ///
    /// # Solidity Signature
    ///
    /// ```solidity
    /// function transfer(address to, uint256 value) external returns (bool);
    /// ```
    #[ink(message)]
    fn transfer(&mut self, to: Address, value: U256) -> bool;

    /// Returns the allowance for a spender on behalf of an owner.
    ///
    /// This shows how many tokens `spender` is allowed to spend on behalf of `owner`.
    ///
    /// # Arguments
    /// * `owner` - The token owner's address
    /// * `spender` - The spender's address
    ///
    /// # Solidity Signature
    ///
    /// ```solidity
    /// function allowance(address owner, address spender) external view returns (uint256);
    /// ```
    #[ink(message)]
    fn allowance(&self, owner: Address, spender: Address) -> U256;

    /// Approves a spender to spend tokens on behalf of the caller.
    ///
    /// # Arguments
    /// * `spender` - The address authorized to spend tokens
    /// * `value` - The maximum amount the spender can spend
    ///
    /// # Returns
    ///
    /// Returns `true` if the approval was successful.
    ///
    /// # Solidity Signature
    ///
    /// ```solidity
    /// function approve(address spender, uint256 value) external returns (bool);
    /// ```
    #[ink(message)]
    fn approve(&mut self, spender: Address, value: U256) -> bool;

    /// Transfers tokens from one account to another using allowance.
    ///
    /// The caller must have sufficient allowance from the `from` account.
    ///
    /// # Arguments
    /// * `from` - The address to transfer tokens from
    /// * `to` - The recipient address
    /// * `value` - The amount of tokens to transfer
    ///
    /// # Returns
    ///
    /// Returns `true` if the transfer was successful.
    ///
    /// # Solidity Signature
    ///
    /// ```solidity
    /// function transferFrom(address from, address to, uint256 value) external returns (bool);
    /// ```
    #[ink(message)]
    #[allow(non_snake_case)]
    fn transferFrom(&mut self, from: Address, to: Address, value: U256) -> bool;
}

/// Creates a new ERC-20 precompile reference for the given asset ID.
///
/// # Arguments
/// * `asset_id` - The ID of the asset to interact with
///
/// # Returns
///
/// Returns an `Erc20Ref` that can be used to call precompile methods.
///
/// # Example
///
/// ```rust,ignore
/// use ink_precompiles::erc20::erc20;
///
/// let asset_id = 1;
/// let erc20_ref = erc20(asset_id);
/// let balance = erc20_ref.balanceOf(account);
/// ```
pub fn erc20(precompile_index: u16, asset_id: AssetId) -> Erc20Ref {
    let address = crate::prefixed_address(precompile_index, asset_id);
    address.into()
}

#[cfg(test)]
mod tests {
    use ink::env::Environment;

    #[test]
    fn erc20_precompile_address_format() {
        // ERC20 Assets precompile for asset ID 1 should be at the correct address
        let expected = [
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x20, 0x00, 0x00,
        ];

        let address = crate::prefixed_address(
            ink::env::DefaultEnvironment::TRUST_BACKED_ASSETS_PRECOMPILE_INDEX,
            1,
        );
        let address_bytes: [u8; 20] = address.into();

        assert_eq!(address_bytes, expected);
    }

    #[test]
    fn erc20_precompile_address_for_multiple_assets() {
        // Test asset ID 42
        let address_42 = crate::prefixed_address(
            ink::env::DefaultEnvironment::TRUST_BACKED_ASSETS_PRECOMPILE_INDEX,
            42,
        );
        let bytes_42: [u8; 20] = address_42.into();

        // First 4 bytes should be asset ID (42 = 0x0000002a)
        assert_eq!(&bytes_42[0..4], &[0x00, 0x00, 0x00, 0x2a]);

        // Bytes 16-19 should be precompile index (0x0120)
        assert_eq!(&bytes_42[16..20], &[0x01, 0x20, 0x00, 0x00]);
    }
}
