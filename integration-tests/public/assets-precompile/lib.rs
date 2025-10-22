#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use ink_precompiles::erc20::{
    AssetId,
    erc20,
};

#[ink::contract]
mod asset_hub_precompile {
    use super::{
        AssetId,
        erc20,
    };
    use ink::prelude::string::String;
    use ink_precompiles::erc20::Erc20;

    #[ink(storage)]
    pub struct AssetHubPrecompile {
        asset_id: AssetId,
    }

    impl AssetHubPrecompile {
        /// Creates a new contract instance for a specific asset ID.
        #[ink(constructor, payable)]
        pub fn new(asset_id: AssetId) -> Self {
            Self { asset_id }
        }

        /// Returns the asset ID this contract is configured for.
        #[ink(message)]
        pub fn asset_id(&self) -> AssetId {
            self.asset_id
        }

        /// Gets the total supply by calling the precompile.
        #[ink(message)]
        pub fn total_supply(&self) -> ink::U256 {
            let precompile = erc20(self.asset_id);
            precompile.totalSupply()
        }

        /// Gets the balance of an account.
        #[ink(message)]
        pub fn balance_of(&self, account: ink::Address) -> ink::U256 {
            let precompile = erc20(self.asset_id);
            precompile.balanceOf(account)
        }

        /// Transfers tokens to another account.
        #[ink(message)]
        pub fn transfer(
            &mut self,
            to: ink::Address,
            amount: ink::U256,
        ) -> Result<bool, String> {
            let mut precompile = erc20(self.asset_id);
            Ok(precompile.transfer(to, amount))
        }

        /// Approves a spender.
        #[ink(message)]
        pub fn approve(
            &mut self,
            spender: ink::Address,
            amount: ink::U256,
        ) -> Result<bool, String> {
            let mut precompile = erc20(self.asset_id);
            Ok(precompile.approve(spender, amount))
        }

        /// Gets the allowance for a spender.
        #[ink(message)]
        pub fn allowance(&self, owner: ink::Address, spender: ink::Address) -> ink::U256 {
            let precompile = erc20(self.asset_id);
            precompile.allowance(owner, spender)
        }

        /// Transfers tokens from one account to another using allowance.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: ink::Address,
            to: ink::Address,
            amount: ink::U256,
        ) -> Result<bool, String> {
            let mut precompile = erc20(self.asset_id);
            Ok(precompile.transferFrom(from, to, amount))
        }
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
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use ink_e2e::ContractsBackend;
    use ink_sandbox::{
        Sandbox,
        api::prelude::{
            AssetsAPI,
            ContractAPI,
        },
    };

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = ink_sandbox::DefaultSandbox,
        client  = ink_sandbox::SandboxClient
    )))]
    async fn deployment_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let mut constructor = asset_hub_precompile::AssetHubPrecompileRef::new(asset_id);

        let contract = client
            .instantiate("assets_precompile", &ink_e2e::alice(), &mut constructor)
            .value(1_000_000_000_000u128) // Transfer native tokens to contract
            .submit()
            .await
            .expect("instantiate failed");

        let call_builder =
            contract.call_builder::<asset_hub_precompile::AssetHubPrecompile>();
        let asset_id_call = call_builder.asset_id();
        let result = client
            .call(&ink_e2e::alice(), &asset_id_call)
            .dry_run()
            .await?;

        assert_eq!(result.return_value(), asset_id);

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = ink_sandbox::DefaultSandbox,
        client  = ink_sandbox::SandboxClient
    )))]
    async fn total_supply_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let admin = ink_sandbox::alice();

        client
            .sandbox()
            .create(&asset_id, &admin, 1u128)
            .expect("Failed to create asset");
        client
            .sandbox()
            .mint_into(&asset_id, &admin, 1000u128)
            .expect("Failed to mint asset");

        let mut constructor = asset_hub_precompile::AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let call_builder =
            contract.call_builder::<asset_hub_precompile::AssetHubPrecompile>();

        let total_supply = call_builder.total_supply();
        let result = client
            .call(&ink_e2e::alice(), &total_supply)
            .submit()
            .await?;

        let supply = result.return_value();
        assert_eq!(supply, ink::U256::from(1000));

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = ink_sandbox::DefaultSandbox,
        client  = ink_sandbox::SandboxClient
    )))]
    async fn balance_of_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let alice = ink_sandbox::alice();
        let bob = ink_sandbox::bob();

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

        let mut constructor = asset_hub_precompile::AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let call_builder =
            contract.call_builder::<asset_hub_precompile::AssetHubPrecompile>();

        let alice_address = ink_e2e::address_from_account_id(&alice);
        let bob_address = ink_e2e::address_from_account_id(&bob);

        // Map bob's account so the precompile can find his assets.
        // (alice is already mapped during instantiate)
        let bob_origin =
            ink_sandbox::DefaultSandbox::convert_account_to_origin(bob.clone());
        client
            .sandbox()
            .map_account(bob_origin)
            .expect("Failed to map bob's account");

        let alice_balance_call = call_builder.balance_of(alice_address);
        let alice_result = client
            .call(&ink_e2e::alice(), &alice_balance_call)
            .dry_run()
            .await?;
        let alice_balance = alice_result.return_value();

        let bob_balance_call = call_builder.balance_of(bob_address);
        let bob_result = client
            .call(&ink_e2e::alice(), &bob_balance_call)
            .dry_run()
            .await?;
        let bob_balance = bob_result.return_value();

        assert_eq!(alice_balance, ink::U256::from(1000));
        assert_eq!(bob_balance, ink::U256::from(500));

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = ink_sandbox::DefaultSandbox,
        client  = ink_sandbox::SandboxClient
    )))]
    async fn transfer_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let alice = ink_sandbox::alice();
        let bob = ink_sandbox::bob();

        client
            .sandbox()
            .create(&asset_id, &alice, 1u128)
            .expect("Failed to create asset");

        let mut constructor = asset_hub_precompile::AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder =
            contract.call_builder::<asset_hub_precompile::AssetHubPrecompile>();

        let bob_address = ink_e2e::address_from_account_id(&bob);
        let transfer_amount = ink::U256::from(1_000);

        // Get contract's AccountId32 - the precompile will see the contract as caller!
        let contract_account =
            ink_sandbox::account_id_from_contract(&contract.account_id);
        client
            .sandbox()
            .mint_into(&asset_id, &contract_account, 100_000u128)
            .expect("Failed to mint to contract");

        let bob_origin =
            ink_sandbox::DefaultSandbox::convert_account_to_origin(bob.clone());
        client
            .sandbox()
            .map_account(bob_origin)
            .expect("Failed to map bob's account");

        let transfer = call_builder.transfer(bob_address, transfer_amount);
        let result = client.call(&ink_e2e::alice(), &transfer).submit().await?;

        let transfer_result = result.return_value();
        assert!(transfer_result.is_ok());
        let contract_balance = client.sandbox().balance_of(&asset_id, &contract_account);
        let bob_balance = client.sandbox().balance_of(&asset_id, &bob);
        assert_eq!(contract_balance, 99_000u128); // Contract had 100_000, transferred 1_000
        assert_eq!(bob_balance, 1_000u128); // Bob received 1_000

        // Show error case with transferring too many tokens.
        let transfer = call_builder.transfer(bob_address, ink::U256::from(1_000_000));
        let result = client.call(&ink_e2e::alice(), &transfer).submit().await?;
        assert_eq!(result.extract_error(), Some("BalanceLow".to_string()));

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = ink_sandbox::DefaultSandbox,
        client  = ink_sandbox::SandboxClient
    )))]
    async fn approve_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let alice = ink_sandbox::alice();
        let bob = ink_sandbox::bob();

        client
            .sandbox()
            .create(&asset_id, &alice, 1u128)
            .expect("Failed to create asset");

        let mut constructor = asset_hub_precompile::AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &ink_e2e::alice(), &mut constructor)
            // Contract needs native balance for approval deposit.
            .value(100_000)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder =
            contract.call_builder::<asset_hub_precompile::AssetHubPrecompile>();

        let contract_account =
            ink_sandbox::account_id_from_contract(&contract.account_id);
        client
            .sandbox()
            .mint_into(&asset_id, &contract_account, 100_000u128)
            .expect("Failed to mint to contract");

        let bob_address = ink_e2e::address_from_account_id(&bob);
        let approve_amount = ink::U256::from(200);

        let bob_origin =
            ink_sandbox::DefaultSandbox::convert_account_to_origin(bob.clone());
        client
            .sandbox()
            .map_account(bob_origin)
            .expect("Failed to map bob's account");

        let bob_allowance_before =
            client
                .sandbox()
                .allowance(&asset_id, &contract_account, &bob);
        assert_eq!(bob_allowance_before, 0u128); // Bob's allowance is 0

        let approve = call_builder.approve(bob_address, approve_amount);
        let result = client.call(&ink_e2e::alice(), &approve).submit().await?;

        assert!(result.return_value().is_ok());
        let bob_allowance =
            client
                .sandbox()
                .allowance(&asset_id, &contract_account, &bob);
        assert_eq!(bob_allowance, 200u128); // Bob's allowance is 200

        Ok(())
    }

    #[ink_sandbox::test(backend(runtime_only(
        sandbox = ink_sandbox::DefaultSandbox,
        client  = ink_sandbox::SandboxClient
    )))]
    async fn allowance_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let alice = ink_sandbox::alice();
        let bob = ink_sandbox::bob();

        client
            .sandbox()
            .create(&asset_id, &alice, 1u128)
            .expect("Failed to create asset");

        let mut constructor = asset_hub_precompile::AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let call_builder =
            contract.call_builder::<asset_hub_precompile::AssetHubPrecompile>();

        client
            .sandbox()
            .mint_into(&asset_id, &alice, 100_000u128)
            .expect("Failed to mint to bob");

        let alice_address = ink_e2e::address_from_account_id(&alice);
        let bob_address = ink_e2e::address_from_account_id(&bob);
        let bob_origin =
            ink_sandbox::DefaultSandbox::convert_account_to_origin(bob.clone());
        client
            .sandbox()
            .map_account(bob_origin)
            .expect("Failed to map bob's account");

        let allowance_call = call_builder.allowance(alice_address, bob_address);
        let result = client
            .call(&ink_e2e::alice(), &allowance_call)
            .dry_run()
            .await?;
        let allowance_before = result.return_value();

        assert_eq!(allowance_before, ink::U256::from(0));

        // Approve bob to spend alice's tokens
        client
            .sandbox()
            .approve(&asset_id, &alice, &bob, 300u128)
            .expect("Failed to approve");

        let result = client
            .call(&ink_e2e::alice(), &allowance_call)
            .dry_run()
            .await?;
        let allowance_after = result.return_value();

        assert_eq!(allowance_after, ink::U256::from(300));

        Ok(())
    }

    /// Tests transferFrom functionality.
    #[ink_sandbox::test(backend(runtime_only(
        sandbox = ink_sandbox::DefaultSandbox,
        client  = ink_sandbox::SandboxClient
    )))]
    async fn transfer_from_works<Client: E2EBackend>(
        mut client: Client,
    ) -> E2EResult<()> {
        let asset_id: u32 = 1;
        let alice = ink_sandbox::alice();
        let bob = ink_sandbox::bob();
        let charlie = ink_sandbox::charlie();

        client
            .sandbox()
            .create(&asset_id, &alice, 1u128)
            .expect("Failed to create asset");

        let mut constructor = asset_hub_precompile::AssetHubPrecompileRef::new(asset_id);
        let contract = client
            .instantiate("assets_precompile", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder =
            contract.call_builder::<asset_hub_precompile::AssetHubPrecompile>();

        let contract_account =
            ink_sandbox::account_id_from_contract(&contract.account_id);
        client
            .sandbox()
            .mint_into(&asset_id, &bob, 100_000u128)
            .expect("Failed to mint to contract");

        // Approve bob to spend contract's tokens
        client
            .sandbox()
            .approve(&asset_id, &bob, &contract_account, 50_000u128)
            .expect("Failed to approve");

        let bob_address = ink_e2e::address_from_account_id(&bob);
        let charlie_address = ink_e2e::address_from_account_id(&charlie);
        let transfer_amount = ink::U256::from(1_500);

        let bob_origin =
            ink_sandbox::DefaultSandbox::convert_account_to_origin(bob.clone());
        client
            .sandbox()
            .map_account(bob_origin)
            .expect("Failed to map bob's account");

        let charlie_origin =
            ink_sandbox::DefaultSandbox::convert_account_to_origin(charlie.clone());
        client
            .sandbox()
            .map_account(charlie_origin)
            .expect("Failed to map charlie's account");

        let transfer_from =
            call_builder.transfer_from(bob_address, charlie_address, transfer_amount);
        let result = client
            .call(&ink_e2e::bob(), &transfer_from)
            .submit()
            .await?;

        assert!(result.return_value().is_ok());

        let bob_balance = client.sandbox().balance_of(&asset_id, &bob);
        let charlie_balance = client.sandbox().balance_of(&asset_id, &charlie);
        let contract_allowance =
            client
                .sandbox()
                .allowance(&asset_id, &bob, &contract_account);

        assert_eq!(bob_balance, 98_500u128); // 100_000 - 1_500
        assert_eq!(charlie_balance, 1_500u128);
        assert_eq!(contract_allowance, 48_500u128);

        // Show error case with transferring more tokens than approved.
        let transfer_from = call_builder.transfer_from(
            bob_address,
            charlie_address,
            ink::U256::from(1_000_000),
        );
        let result = client
            .call(&ink_e2e::bob(), &transfer_from)
            .submit()
            .await?;
        assert_eq!(result.extract_error(), Some("Unapproved".to_string()));

        Ok(())
    }
}
