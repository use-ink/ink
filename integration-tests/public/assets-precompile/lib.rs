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
mod tests {
    use super::*;

    #[test]
    fn contract_stores_asset_id() {
        use asset_hub_precompile::AssetHubPrecompile;

        let contract = AssetHubPrecompile::new(1337);

        assert_eq!(contract.asset_id(), 1337);
    }

    #[test]
    fn contract_stores_owner() {
        use asset_hub_precompile::AssetHubPrecompile;

        let contract = AssetHubPrecompile::new(1337);

        assert_eq!(contract.asset_id(), 1337);
        // Note: In unit tests, the caller is always the zero address
        assert_eq!(contract.owner(), H160::from([0u8; 20]));
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use crate::asset_hub_precompile::{
        Approval,
        AssetHubPrecompile,
        AssetHubPrecompileRef,
        Transfer,
    };
    use ink_e2e::{
        ContractsBackend,
        IntoAddress,
        alice,
        bob,
    };
    use ink_sandbox::{
        DefaultSandbox,
        E2EError,
        SandboxClient,
        api::prelude::{
            AssetsAPI,
            ContractAPI,
        },
        assert_last_event,
        assert_noop,
        assert_ok,
    };

    type E2EResult<T> = std::result::Result<T, E2EError>;

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn deployment_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let mut constructor = AssetHubPrecompileRef::new(asset_id);

        let contract = client
            .instantiate("assets_precompile", &alice(), &mut constructor)
            .value(1_000_000_000_000u128) // Transfer native tokens to contract
            .submit()
            .await?;

        let contract_call = contract.call_builder::<AssetHubPrecompile>();
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
        let admin = alice();

        client.sandbox().create(&asset_id, &admin, 1u128)?;
        client.sandbox().mint_into(&asset_id, &admin, 1000u128)?;

        let contract = client
            .instantiate(
                "assets_precompile",
                &admin,
                &mut AssetHubPrecompileRef::new(asset_id),
            )
            .submit()
            .await?;

        let contract_call = contract.call_builder::<AssetHubPrecompile>();
        let result = client
            .call(&admin, &contract_call.total_supply())
            .submit()
            .await?;

        assert_eq!(result.return_value(), U256::from(1000));
        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn balance_of_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let alice = alice();
        let bob = bob();

        client.sandbox().create(&asset_id, &alice, 1u128)?;
        client.sandbox().mint_into(&asset_id, &alice, 1000u128)?;
        client.sandbox().mint_into(&asset_id, &bob, 500u128)?;

        let contract = client
            .instantiate(
                "assets_precompile",
                &alice,
                &mut AssetHubPrecompileRef::new(asset_id),
            )
            .submit()
            .await?;

        // Map bob's account otherwise it fails.
        client.sandbox().map_account(&bob)?;

        let contract_call = contract.call_builder::<AssetHubPrecompile>();
        let alice_balance = client
            .call(&alice, &contract_call.balance_of(alice.address()))
            .dry_run()
            .await?;
        assert_eq!(alice_balance.return_value(), U256::from(1000));
        let bob_balance = client
            .call(&alice, &contract_call.balance_of(bob.address()))
            .dry_run()
            .await?;
        assert_eq!(bob_balance.return_value(), U256::from(500));

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn transfer_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let alice = alice();
        let bob = bob();

        client.sandbox().create(&asset_id, &alice, 1u128)?;

        let contract = client
            .instantiate(
                "assets_precompile",
                &alice,
                &mut AssetHubPrecompileRef::new(asset_id),
            )
            .submit()
            .await?;

        client
            .sandbox()
            .mint_into(&asset_id, &contract.account_id, 100_000u128)?;
        client.sandbox().map_account(&bob)?;

        let mut contract_call = contract.call_builder::<AssetHubPrecompile>();
        let bob_address = bob.address();
        let transfer_amount = U256::from(1_000);

        let result = client
            .call(
                &alice,
                &contract_call.transfer(bob_address, transfer_amount),
            )
            .submit()
            .await?;
        assert_ok!(result);
        assert_last_event!(
            &mut client,
            Transfer {
                from: contract.addr,
                to: bob_address,
                value: transfer_amount
            }
        );

        let contract_balance =
            client.sandbox().balance_of(&asset_id, &contract.account_id);
        let bob_balance = client.sandbox().balance_of(&asset_id, &bob);
        assert_eq!(contract_balance, 99_000u128);
        assert_eq!(bob_balance, 1_000u128);

        let result = client
            .call(
                &alice,
                &contract_call.transfer(bob_address, U256::from(1_000_000)),
            )
            .submit()
            .await?;
        assert_noop!(result, "BalanceLow");

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn approve_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let alice = alice();
        let bob = bob();

        client.sandbox().create(&asset_id, &alice, 1u128)?;

        let contract = client
            .instantiate("assets_precompile", &alice, &mut AssetHubPrecompileRef::new(asset_id))
            // Contract needs native balance for approval deposit.
            .value(100_000)
            .submit()
            .await?;

        client
            .sandbox()
            .mint_into(&asset_id, &contract.account_id, 100_000u128)?;
        client.sandbox().map_account(&bob)?;
        let bob_allowance_before =
            client
                .sandbox()
                .allowance(&asset_id, &contract.account_id, &bob);
        assert_eq!(bob_allowance_before, 0u128); // Bob's allowance is 0

        let mut contract_call = contract.call_builder::<AssetHubPrecompile>();
        let bob_address = bob.address();
        let approve_amount = U256::from(200);

        let result = client
            .call(&alice, &contract_call.approve(bob_address, approve_amount))
            .submit()
            .await?;
        assert_ok!(result);
        assert_last_event!(
            &mut client,
            Approval {
                owner: contract.addr,
                spender: bob_address,
                value: approve_amount,
            }
        );

        let bob_allowance =
            client
                .sandbox()
                .allowance(&asset_id, &contract.account_id, &bob);
        assert_eq!(bob_allowance, 200u128);

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn allowance_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let alice = alice();
        let bob = bob();

        client.sandbox().create(&asset_id, &alice, 1u128)?;

        let contract = client
            .instantiate(
                "assets_precompile",
                &alice,
                &mut AssetHubPrecompileRef::new(asset_id),
            )
            .submit()
            .await?;

        let contract_call = contract.call_builder::<AssetHubPrecompile>();
        client.sandbox().mint_into(&asset_id, &alice, 100_000u128)?;
        client.sandbox().map_account(&bob)?;

        let allowance_call = &contract_call.allowance(alice.address(), bob.address());
        let result = client.call(&alice, allowance_call).dry_run().await?;
        assert_eq!(result.return_value(), U256::from(0));

        // Approve bob to spend alice's tokens
        client.sandbox().approve(&asset_id, &alice, &bob, 300u128)?;

        let result = client.call(&alice, allowance_call).dry_run().await?;
        assert_eq!(result.return_value(), U256::from(300));

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = DefaultSandbox,
        client  = SandboxClient
    )))]
    async fn transfer_from_works<Client: E2EBackend>(
        mut client: Client,
    ) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let alice = alice();
        let bob = bob();

        client.sandbox().create(&asset_id, &alice, 1u128)?;

        let contract = client
            .instantiate(
                "assets_precompile",
                &alice,
                &mut AssetHubPrecompileRef::new(asset_id),
            )
            .submit()
            .await?;

        client.sandbox().mint_into(&asset_id, &alice, 100_000u128)?;
        // Approve alice to spend contract's tokens
        client
            .sandbox()
            .approve(&asset_id, &alice, &contract.account_id, 50_000u128)?;
        client.sandbox().map_account(&bob)?;

        let mut contract_call = contract.call_builder::<AssetHubPrecompile>();
        let alice_address = alice.address();
        let bob_address = bob.address();
        let transfer_amount = U256::from(1_500);
        let result = client
            .call(
                &alice,
                &contract_call.transfer_from(alice_address, bob_address, transfer_amount),
            )
            .submit()
            .await?;
        assert_ok!(result);
        assert_last_event!(
            &mut client,
            Transfer {
                from: alice_address,
                to: bob_address,
                value: transfer_amount,
            }
        );

        let alice_balance = client.sandbox().balance_of(&asset_id, &alice);
        let bob_balance = client.sandbox().balance_of(&asset_id, &bob);
        let contract_allowance =
            client
                .sandbox()
                .allowance(&asset_id, &alice, &contract.account_id);
        assert_eq!(alice_balance, 98_500u128);
        assert_eq!(bob_balance, 1_500u128);
        assert_eq!(contract_allowance, 48_500u128);

        let result = client
            .call(
                &alice,
                &contract_call.transfer_from(
                    alice_address,
                    bob_address,
                    U256::from(1_000_000),
                ),
            )
            .submit()
            .await?;
        assert_noop!(result, "Unapproved");
        Ok(())
    }
}
