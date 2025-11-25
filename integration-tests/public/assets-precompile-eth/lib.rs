//! # Assets Precompile Integration Test (Ethereum-First Approach)
//!
//! This integration test demonstrates the recommended approach for interacting with
//! pallet-revive contracts using **Ethereum addresses as the default**.
//!
//! ## Why Ethereum Addresses?
//!
//! When using Substrate addresses (AccountId32) with pallet-revive, you encounter
//! the "fallback account trap":
//!
//! 1. Substrate AccountId32 → H160 conversion uses `keccak256(AccountId32)[12..]`
//! 2. When converting back (H160 → AccountId32), if the account isn't explicitly mapped,
//!    tokens go to a "fallback account" (`H160 + 0xEE padding`)
//! 3. This fallback account is NOT the original Substrate account!
//!
//! ## The Ethereum-First Solution
//!
//! By using Ethereum addresses (H160) directly:
//!
//! 1. The H160 address is the native Ethereum address
//! 2. AccountId32 uses the fallback format: `[H160][0xEE; 12]`
//! 3. `is_eth_derived()` returns TRUE → automatically "mapped"
//! 4. Perfect roundtrip: H160 → AccountId32 → H160
//!
//! **No `map_account()` calls needed!**
//!
//! ## Note on Test Infrastructure
//!
//! While the contract parameters use Ethereum addresses, the current test
//! infrastructure still uses Sr25519 keypairs for transaction signing.
//! The important point is that the **addresses passed to contracts** are
//! Ethereum addresses, which ensures correct token routing.

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
mod asset_hub_precompile_eth {
    use super::*;
    use ink::prelude::string::String;
    use ink_precompiles::erc20::{
        Erc20,
        Erc20Ref,
    };

    #[ink(storage)]
    pub struct AssetHubPrecompileEth {
        asset_id: AssetId,
        /// The owner of this contract. Only the owner can call transfer, approve, and
        /// transfer_from. This is necessary because the contract holds tokens
        /// and without access control, anyone could transfer tokens that the
        /// contract holds, which would be a security issue.
        owner: H160,
        precompile: Erc20Ref,
    }

    impl AssetHubPrecompileEth {
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
mod tests {
    use super::*;

    #[test]
    fn contract_stores_asset_id() {
        use asset_hub_precompile_eth::AssetHubPrecompileEth;

        let contract = AssetHubPrecompileEth::new(1337);

        assert_eq!(contract.asset_id(), 1337);
    }

    #[test]
    fn contract_stores_owner() {
        use asset_hub_precompile_eth::AssetHubPrecompileEth;

        let contract = AssetHubPrecompileEth::new(1337);

        assert_eq!(contract.asset_id(), 1337);
        // Note: In unit tests, the caller is always the zero address
        assert_eq!(contract.owner(), H160::from([0u8; 20]));
    }
}

/// End-to-end tests demonstrating the Ethereum-first approach.
///
/// ## Key Concepts Demonstrated
///
/// 1. **Ethereum addresses for token recipients**: When sending tokens to users, we use
///    Ethereum H160 addresses directly. This ensures the tokens go to accounts that can
///    be controlled via MetaMask.
///
/// 2. **Fallback account format**: Ethereum addresses are stored as AccountId32 using the
///    fallback format: `[H160][0xEE; 12]`. This format is automatically recognized as
///    "Ethereum-derived" by pallet-revive.
///
/// 3. **No mapping required**: Unlike Substrate addresses, Ethereum addresses don't need
///    explicit mapping. The `is_eth_derived()` check passes automatically.
///
/// ## Test Infrastructure Note
///
/// The current test infrastructure uses Sr25519 keypairs for transaction signing.
/// However, the important point is that **contract parameters use Ethereum addresses**,
/// which ensures correct token routing without the fallback account trap.
#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use crate::asset_hub_precompile_eth::{
        Approval,
        AssetHubPrecompileEth,
        AssetHubPrecompileEthRef,
        Transfer,
    };
    use ink_e2e::{
        ContractsBackend,
        IntoAddress,
        // Sr25519 keypairs for test infrastructure signing
        alice,
        // Ethereum keypairs for addresses
        eth::{
            alith,
            baltathar,
        },
    };
    use ink_sandbox::{
        AccountId32,
        DefaultSandbox,
        E2EError,
        IntoAccountId,
        SandboxClient,
        api::prelude::{
            AssetsAPI,
            BalanceAPI,
        },
        assert_last_event,
        assert_ok,
    };

    type E2EResult<T> = std::result::Result<T, E2EError>;

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn deployment_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let mut constructor = AssetHubPrecompileEthRef::new(asset_id);

        // Use alice() for signing, but the contract will be owned by alice's derived
        // address
        let contract = client
            .instantiate("assets_precompile_eth", &alice(), &mut constructor)
            .value(1_000_000_000_000u128)
            .submit()
            .await?;

        let contract_call = contract.call_builder::<AssetHubPrecompileEth>();
        let result = client
            .call(&alice(), &contract_call.asset_id())
            .dry_run()
            .await?;

        assert_eq!(result.return_value(), asset_id);

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn total_supply_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;

        // Create and mint using Ethereum keypairs (works with IntoAccountId)
        client.sandbox().create(&asset_id, &alith(), 1u128)?;
        AssetsAPI::mint_into(client.sandbox(), &asset_id, &alith(), 1000u128)?;

        let contract = client
            .instantiate(
                "assets_precompile_eth",
                &alice(),
                &mut AssetHubPrecompileEthRef::new(asset_id),
            )
            .submit()
            .await?;

        let contract_call = contract.call_builder::<AssetHubPrecompileEth>();
        let result = client
            .call(&alice(), &contract_call.total_supply())
            .submit()
            .await?;

        assert_eq!(result.return_value(), U256::from(1000));
        Ok(())
    }

    /// This test demonstrates using Ethereum addresses for querying balances.
    ///
    /// Key point: We use `alith().address()` and `baltathar().address()` to get
    /// the native Ethereum H160 addresses, which work correctly with the precompile.
    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn balance_of_with_eth_addresses<Client: E2EBackend>(
        mut client: Client,
    ) -> E2EResult<()> {
        let asset_id: u32 = 1;

        // Create asset with alith as admin
        client.sandbox().create(&asset_id, &alith(), 1u128)?;

        // Mint to Ethereum addresses (uses IntoAccountId for EthKeypair)
        AssetsAPI::mint_into(client.sandbox(), &asset_id, &alith(), 1000u128)?;
        AssetsAPI::mint_into(client.sandbox(), &asset_id, &baltathar(), 500u128)?;

        let contract = client
            .instantiate(
                "assets_precompile_eth",
                &alice(),
                &mut AssetHubPrecompileEthRef::new(asset_id),
            )
            .submit()
            .await?;

        let contract_call = contract.call_builder::<AssetHubPrecompileEth>();

        // Query balance using Ethereum addresses directly
        // This is the key point: we use alith().address() which returns the native H160
        let alith_balance = client
            .call(&alice(), &contract_call.balance_of(alith().address()))
            .dry_run()
            .await?;
        assert_eq!(alith_balance.return_value(), U256::from(1000));

        let baltathar_balance = client
            .call(&alice(), &contract_call.balance_of(baltathar().address()))
            .dry_run()
            .await?;
        assert_eq!(baltathar_balance.return_value(), U256::from(500));

        Ok(())
    }

    /// Transfer test using Ethereum addresses for recipients.
    ///
    /// This demonstrates that tokens sent to Ethereum addresses can be
    /// correctly tracked without explicit account mapping.
    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn transfer_to_eth_address<Client: E2EBackend>(
        mut client: Client,
    ) -> E2EResult<()> {
        let asset_id: u32 = 1;

        client.sandbox().create(&asset_id, &alice(), 1u128)?;

        let contract = client
            .instantiate(
                "assets_precompile_eth",
                &alice(),
                &mut AssetHubPrecompileEthRef::new(asset_id),
            )
            .submit()
            .await?;

        // Mint tokens to the contract
        AssetsAPI::mint_into(
            client.sandbox(),
            &asset_id,
            &contract.account_id,
            100_000u128,
        )?;

        let mut contract_call = contract.call_builder::<AssetHubPrecompileEth>();

        // Transfer to an Ethereum address (baltathar)
        // The key point: baltathar().address() is a native H160 that will work
        // correctly without mapping
        let baltathar_addr = baltathar().address();
        let transfer_amount = U256::from(1_000);

        let result = client
            .call(
                &alice(),
                &contract_call.transfer(baltathar_addr, transfer_amount),
            )
            .submit()
            .await?;
        assert_ok!(result);
        assert_last_event!(
            &mut client,
            Transfer {
                from: contract.addr,
                to: baltathar_addr,
                value: transfer_amount
            }
        );

        // Verify balances using sandbox (which uses IntoAccountId for EthKeypair)
        let contract_balance =
            client.sandbox().balance_of(&asset_id, &contract.account_id);
        let baltathar_balance = client.sandbox().balance_of(&asset_id, &baltathar());
        assert_eq!(contract_balance, 99_000u128);
        assert_eq!(baltathar_balance, 1_000u128);

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn approve_with_eth_address<Client: E2EBackend>(
        mut client: Client,
    ) -> E2EResult<()> {
        let asset_id: u32 = 1;

        client.sandbox().create(&asset_id, &alice(), 1u128)?;

        let contract = client
            .instantiate(
                "assets_precompile_eth",
                &alice(),
                &mut AssetHubPrecompileEthRef::new(asset_id),
            )
            .value(100_000)
            .submit()
            .await?;

        AssetsAPI::mint_into(
            client.sandbox(),
            &asset_id,
            &contract.account_id,
            100_000u128,
        )?;

        // Check initial allowance is 0
        let baltathar_allowance_before =
            client
                .sandbox()
                .allowance(&asset_id, &contract.account_id, &baltathar());
        assert_eq!(baltathar_allowance_before, 0u128);

        let mut contract_call = contract.call_builder::<AssetHubPrecompileEth>();

        // Approve baltathar (using Ethereum address)
        let baltathar_addr = baltathar().address();
        let approve_amount = U256::from(200);

        let result = client
            .call(
                &alice(),
                &contract_call.approve(baltathar_addr, approve_amount),
            )
            .submit()
            .await?;
        assert_ok!(result);
        assert_last_event!(
            &mut client,
            Approval {
                owner: contract.addr,
                spender: baltathar_addr,
                value: approve_amount,
            }
        );

        // Verify allowance using sandbox
        let baltathar_allowance =
            client
                .sandbox()
                .allowance(&asset_id, &contract.account_id, &baltathar());
        assert_eq!(baltathar_allowance, 200u128);

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn allowance_query_with_eth_addresses<Client: E2EBackend>(
        mut client: Client,
    ) -> E2EResult<()> {
        let asset_id: u32 = 1;

        // Fund the alith account with native tokens (needed for transaction fees)
        let alith_account: AccountId32 = (&alith()).into_account_id();
        BalanceAPI::mint_into(client.sandbox(), &alith_account, 1_000_000_000_000u128)?;

        client.sandbox().create(&asset_id, &alith(), 1u128)?;

        let contract = client
            .instantiate(
                "assets_precompile_eth",
                &alice(),
                &mut AssetHubPrecompileEthRef::new(asset_id),
            )
            .submit()
            .await?;

        let contract_call = contract.call_builder::<AssetHubPrecompileEth>();
        AssetsAPI::mint_into(client.sandbox(), &asset_id, &alith(), 100_000u128)?;

        // Query allowance using Ethereum addresses
        let allowance_call =
            &contract_call.allowance(alith().address(), baltathar().address());
        let result = client.call(&alice(), allowance_call).dry_run().await?;
        assert_eq!(result.return_value(), U256::from(0));

        // Approve using sandbox (which uses IntoAccountId for EthKeypair)
        client
            .sandbox()
            .approve(&asset_id, &alith(), &baltathar(), 300u128)?;

        let result = client.call(&alice(), allowance_call).dry_run().await?;
        assert_eq!(result.return_value(), U256::from(300));

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn transfer_from_with_eth_addresses<Client: E2EBackend>(
        mut client: Client,
    ) -> E2EResult<()> {
        let asset_id: u32 = 1;

        // Fund the alith account with native tokens (needed for transaction fees)
        let alith_account: AccountId32 = (&alith()).into_account_id();
        BalanceAPI::mint_into(client.sandbox(), &alith_account, 1_000_000_000_000u128)?;

        client.sandbox().create(&asset_id, &alith(), 1u128)?;

        let contract = client
            .instantiate(
                "assets_precompile_eth",
                &alice(),
                &mut AssetHubPrecompileEthRef::new(asset_id),
            )
            .submit()
            .await?;

        // Mint to alith and approve the contract to spend
        AssetsAPI::mint_into(client.sandbox(), &asset_id, &alith(), 100_000u128)?;
        client.sandbox().approve(
            &asset_id,
            &alith(),
            &contract.account_id,
            50_000u128,
        )?;

        let mut contract_call = contract.call_builder::<AssetHubPrecompileEth>();

        // Transfer from alith to baltathar (both Ethereum addresses)
        let alith_addr = alith().address();
        let baltathar_addr = baltathar().address();
        let transfer_amount = U256::from(1_500);

        let result = client
            .call(
                &alice(),
                &contract_call.transfer_from(alith_addr, baltathar_addr, transfer_amount),
            )
            .submit()
            .await?;
        assert_ok!(result);
        assert_last_event!(
            &mut client,
            Transfer {
                from: alith_addr,
                to: baltathar_addr,
                value: transfer_amount,
            }
        );

        // Verify balances using sandbox
        let alith_balance = client.sandbox().balance_of(&asset_id, &alith());
        let baltathar_balance = client.sandbox().balance_of(&asset_id, &baltathar());
        let contract_allowance =
            client
                .sandbox()
                .allowance(&asset_id, &alith(), &contract.account_id);
        assert_eq!(alith_balance, 98_500u128);
        assert_eq!(baltathar_balance, 1_500u128);
        assert_eq!(contract_allowance, 48_500u128);

        Ok(())
    }
}
