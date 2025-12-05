#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    H160,
    U256,
    prelude::string::ToString,
};
pub use ink_precompiles::erc20::{
    AssetId,
    erc20,
};

#[ink::contract]
mod asset_hub_precompile {
    use super::*;
    use ink::prelude::string::String;
    use ink_precompiles::erc20::{
        Erc20,
        Erc20Ref,
    };

    #[ink(storage)]
    pub struct AssetHubPrecompile {
        asset_id: AssetId,
        /// The owner of this contract. Only the owner can call transfer, approve, and
        /// transfer_from. This is necessary because the contract holds tokens
        /// and without access control, anyone could transfer tokens that the
        /// contract holds, which would be a security issue.
        owner: H160,
        precompile: Erc20Ref,
    }

    impl AssetHubPrecompile {
        /// Creates a new contract instance for a specific asset ID.
        #[ink(constructor, payable)]
        pub fn new(asset_id: AssetId) -> Self {
            Self {
                asset_id,
                owner: Self::env().caller(),
                precompile: erc20(TRUST_BACKED_ASSETS_PRECOMPILE_INDEX, asset_id),
            }
        }

        /// Returns the asset ID this contract is configured for.
        #[ink(message)]
        pub fn asset_id(&self) -> AssetId {
            self.asset_id
        }

        /// Returns the owner of this contract.
        #[ink(message)]
        pub fn owner(&self) -> H160 {
            self.owner
        }

        /// Ensures only the owner can call this function.
        fn ensure_owner(&self) -> Result<(), String> {
            if self.env().caller() != self.owner {
                return Err("Only owner can call this function".to_string());
            }
            Ok(())
        }

        /// Gets the total supply by calling the precompile.
        #[ink(message)]
        pub fn total_supply(&self) -> U256 {
            self.precompile.totalSupply()
        }

        /// Gets the balance of an account.
        #[ink(message)]
        pub fn balance_of(&self, account: Address) -> U256 {
            self.precompile.balanceOf(account)
        }

        /// Transfers tokens to another account.
        #[ink(message)]
        pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, String> {
            self.ensure_owner()?;
            if !self.precompile.transfer(to, value) {
                return Err("Transfer failed".to_string());
            }
            self.env().emit_event(Transfer {
                from: self.env().address(),
                to,
                value,
            });
            Ok(true)
        }

        /// Approves a spender.
        #[ink(message)]
        pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, String> {
            self.ensure_owner()?;
            if !self.precompile.approve(spender, value) {
                return Err("Approval failed".to_string());
            }
            self.env().emit_event(Approval {
                owner: self.env().address(),
                spender,
                value,
            });
            Ok(true)
        }

        /// Gets the allowance for a spender.
        #[ink(message)]
        pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
            self.precompile.allowance(owner, spender)
        }

        /// Transfers tokens from one account to another using allowance.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
            value: U256,
        ) -> Result<bool, String> {
            self.ensure_owner()?;
            if !self.precompile.transferFrom(from, to, value) {
                return Err("Transfer failed".to_string());
            }
            self.env().emit_event(Transfer { from, to, value });
            Ok(true)
        }
    }

    /// Event emitted when allowance by `owner` to `spender` changes.
    #[derive(Debug, PartialEq)]
    #[ink::event]
    pub struct Approval {
        #[ink(topic)]
        pub owner: Address,
        #[ink(topic)]
        pub spender: Address,
        pub value: U256,
    }

    /// Event emitted when transfer of tokens occurs.
    #[derive(Debug, PartialEq)]
    #[ink::event]
    pub struct Transfer {
        #[ink(topic)]
        pub from: Address,
        #[ink(topic)]
        pub to: Address,
        pub value: U256,
    }
}

#[cfg(test)]
mod tests;