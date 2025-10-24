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
                precompile: erc20(asset_id),
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
    #[derive(Debug)]
    #[ink::event]
    pub struct Approval {
        #[ink(topic)]
        pub owner: Address,
        #[ink(topic)]
        pub spender: Address,
        pub value: U256,
    }

    /// Event emitted when transfer of tokens occurs.
    #[derive(Debug)]
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
        alice, bob,
    };
    use ink_sandbox::{
        DefaultSandbox,
        SandboxClient,
        api::prelude::{
            AssetsAPI,
            ContractAPI,
        },
        assert_last_contract_event,
    };

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
            .await
            .expect("instantiate failed");

        let call_builder = contract.call_builder::<AssetHubPrecompile>();
        let asset_id_call = call_builder.asset_id();
        let result = client
            .call(&alice(), &asset_id_call)
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

        client
            .sandbox()
            .create(&asset_id, &admin, 1u128)
            .expect("Failed to create asset");
        client
            .sandbox()
            .mint_into(&asset_id, &admin, 1000u128)
            .expect("Failed to mint asset");

        let mut constructor = AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &admin, &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let call_builder =
            contract.call_builder::<AssetHubPrecompile>();

        let total_supply = call_builder.total_supply();
        let result = client
            .call(&admin, &total_supply)
            .submit()
            .await?;

        let supply = result.return_value();
        assert_eq!(supply, U256::from(1000));

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

        client
            .sandbox()
            .create(&asset_id, &alice, 1u128)
            .expect("Failed to create asset");
        client
            .sandbox()
            .mint_into(&asset_id, &alice, 1000u128)
            .expect("Failed to mint to alice");
        client
            .sandbox()
            .mint_into(&asset_id, &bob, 500u128)
            .expect("Failed to mint to bob");

        let mut constructor = AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &alice, &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let call_builder =
            contract.call_builder::<AssetHubPrecompile>();

        // Map bob's account otherwise it fails.
        client
            .sandbox()
            .map_account(&bob)
            .expect("Failed to map bob's account");

        let alice_balance_call = call_builder.balance_of(alice.address());
        let alice_result = client
            .call(&alice, &alice_balance_call)
            .dry_run()
            .await?;
        let alice_balance = alice_result.return_value();

        let bob_balance_call = call_builder.balance_of(bob.address());
        let bob_result = client
            .call(&alice, &bob_balance_call)
            .dry_run()
            .await?;
        let bob_balance = bob_result.return_value();

        assert_eq!(alice_balance, U256::from(1000));
        assert_eq!(bob_balance, U256::from(500));

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

        client
            .sandbox()
            .create(&asset_id, &alice, 1u128)
            .expect("Failed to create asset");

        let mut constructor = AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &alice, &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder =
            contract.call_builder::<AssetHubPrecompile>();

        let bob_address = bob.address();
        let transfer_amount = U256::from(1_000);
        client
            .sandbox()
            .mint_into(&asset_id, &contract.account_id, 100_000u128)
            .expect("Failed to mint to contract");
        client
            .sandbox()
            .map_account(&bob)
            .expect("Failed to map bob's account");

        let transfer = call_builder.transfer(bob_address, transfer_amount);
        let result = client.call(&alice, &transfer).submit().await?;

        let transfer_result = result.return_value();
        assert!(transfer_result.is_ok());
        assert_last_contract_event!(
            &mut client,
            Transfer {
                from: contract.addr,
                to: bob_address,
                value: transfer_amount
            }
        );
        let contract_balance = client.sandbox().balance_of(&asset_id, &contract.account_id);
        let bob_balance = client.sandbox().balance_of(&asset_id, &bob);
        assert_eq!(contract_balance, 99_000u128); // Contract had 100_000, transferred 1_000
        assert_eq!(bob_balance, 1_000u128); // Bob received 1_000

        // Show error case with transferring too many tokens.
        let transfer = call_builder.transfer(bob_address, U256::from(1_000_000));
        let result = client.call(&alice, &transfer).submit().await?;
        assert_eq!(result.extract_error(), Some("BalanceLow".to_string()));

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

        client
            .sandbox()
            .create(&asset_id, &alice, 1u128)
            .expect("Failed to create asset");

        let mut constructor = AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &alice, &mut constructor)
            // Contract needs native balance for approval deposit.
            .value(100_000)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder =
            contract.call_builder::<AssetHubPrecompile>();

        client
            .sandbox()
            .mint_into(&asset_id, &contract.account_id, 100_000u128)
            .expect("Failed to mint to contract");
        let bob_address = bob.address();
        let approve_amount = U256::from(200);

        client
            .sandbox()
            .map_account(&bob)
            .expect("Failed to map bob's account");

        let bob_allowance_before =
            client
                .sandbox()
                .allowance(&asset_id, &contract.account_id, &bob);
        assert_eq!(bob_allowance_before, 0u128); // Bob's allowance is 0

        let approve = call_builder.approve(bob_address, approve_amount);
        let result = client.call(&alice, &approve).submit().await?;

        assert!(result.return_value().is_ok());
        assert_last_contract_event!(
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
        assert_eq!(bob_allowance, 200u128); // Bob's allowance is 200

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

        client
            .sandbox()
            .create(&asset_id, &alice, 1u128)
            .expect("Failed to create asset");

        let mut constructor = AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &alice, &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let call_builder =
            contract.call_builder::<AssetHubPrecompile>();

        client
            .sandbox()
            .mint_into(&asset_id, &alice, 100_000u128)
            .expect("Failed to mint to bob");

        let alice_address = alice.address();
        let bob_address = bob.address();

        client
            .sandbox()
            .map_account(&bob)
            .expect("Failed to map bob's account");

        let allowance_call = call_builder.allowance(alice_address, bob_address);
        let result = client
            .call(&alice, &allowance_call)
            .dry_run()
            .await?;
        let allowance_before = result.return_value();

        assert_eq!(allowance_before, U256::from(0));

        // Approve bob to spend alice's tokens
        client
            .sandbox()
            .approve(&asset_id, &alice, &bob, 300u128)
            .expect("Failed to approve");

        let result = client
            .call(&alice, &allowance_call)
            .dry_run()
            .await?;
        let allowance_after = result.return_value();

        assert_eq!(allowance_after, U256::from(300));

        Ok(())
    }

    /// Tests transferFrom functionality.
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

        client
            .sandbox()
            .create(&asset_id, &alice, 1u128)
            .expect("Failed to create asset");

        let mut constructor = AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &alice, &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder =
            contract.call_builder::<AssetHubPrecompile>();

        client
            .sandbox()
            .mint_into(&asset_id, &alice, 100_000u128)
            .expect("Failed to mint to contract");

        // Approve alice to spend contract's tokens
        client
            .sandbox()
            .approve(&asset_id, &alice, &contract.account_id, 50_000u128)
            .expect("Failed to approve");

        let alice_address = alice.address();
        let bob_address = bob.address();
        let transfer_amount = U256::from(1_500);

        client
            .sandbox()
            .map_account(&bob)
            .expect("Failed to map bob's account");

        let transfer_from =
            call_builder.transfer_from(alice_address, bob_address, transfer_amount);
        let result = client
            .call(&alice, &transfer_from)
            .submit()
            .await?;

        assert!(result.return_value().is_ok());
        assert_last_contract_event!(
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

        assert_eq!(alice_balance, 98_500u128); // 100_000 - 1_500
        assert_eq!(bob_balance, 1_500u128);
        assert_eq!(contract_allowance, 48_500u128);

        // Show error case with transferring more tokens than approved.
        let transfer_from =
            call_builder.transfer_from(alice_address, bob_address, U256::from(1_000_000));
        let result = client
            .call(&alice, &transfer_from)
            .submit()
            .await?;
        assert_eq!(result.extract_error(), Some("Unapproved".to_string()));

        Ok(())
    }
}
