//! The integration test does cross contract calls to the
//! [`other_contract::OtherContract`] via the [`ink::env::call::CallBuilder`].
//! It tests how to call read-only methods, mutable methods, methods that return
//! something, and methods with several input arguments.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod builder_contract_caller {
    use ink::env::call_builder;
    /// We import the generated `ContractRef` of our other contract.
    use other_contract::OtherContractRef;

    #[ink(storage)]
    pub struct BuilderContractCaller {
        /// We specify that our contract will store a reference to the `OtherContract`.
        other_contract: OtherContractRef,
    }

    impl BuilderContractCaller {
        /// In order to use the `OtherContract` we first need to **instantiate** it.
        ///
        /// To do this we will use the uploaded `code_hash` of `OtherContract`.
        #[ink(constructor)]
        pub fn new(other_contract_code_hash: Hash) -> Self {
            let other_contract = OtherContractRef::new()
                .code_hash(other_contract_code_hash)
                .endowment(0)
                .salt_bytes([0xDE, 0xAD, 0xBE, 0xEF])
                .instantiate();

            Self { other_contract }
        }

        /// Return the total supply of the `OtherContract`.
        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            use other_contract::Erc20;
            call_builder!(self.other_contract.total_supply()).invoke()
        }

        /// Return the balance of the `owner` in the `OtherContract`.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u128 {
            call_builder!(<_ as other_contract::Erc20>::balance_of(
                &self.other_contract,
                owner
            ))
            .invoke()
        }

        /// Using the `{Contract}Ref` we can call all the messages of the `OtherContract`
        /// as if they were normal Rust methods (because at the end of the day,
        /// they are!).
        #[ink(message)]
        pub fn mint_and_transfer(&mut self, to: AccountId, amount: u128) {
            // Mint tokens to self.
            call_builder!(self.other_contract.mint(self.env().account_id(), amount))
                .invoke();
            // Transfer tokens from self to `to`.
            call_builder!(other_contract::Erc20::transfer(
                &mut self.other_contract,
                to,
                amount
            ))
            .invoke();
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::builder_contract_caller::{
        BuilderContractCaller,
        BuilderContractCallerRef,
    };
    use ink::primitives::AccountId;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    /// A test deploys and instantiates the
    /// `builder_contract_caller::BuilderContractCaller` contract.
    ///
    /// The test verifies that we can call `BuilderContractCaller::mint_and_transfer` and
    /// `BuilderContractCaller::total_supply`.
    #[ink_e2e::test(additional_contracts = "other_contract/Cargo.toml")]
    async fn e2e_cross_contract_calls(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let other_code_hash = client
            .upload("other-contract", &ink_e2e::alice(), None)
            .await
            .expect("uploading `other-contract` failed")
            .code_hash;

        let _ = client
            .upload("builder-contract-caller", &ink_e2e::alice(), None)
            .await
            .expect("uploading `builder-contract-caller` failed")
            .code_hash;

        let constructor = BuilderContractCallerRef::new(other_code_hash);

        let contract = client
            .instantiate(
                "builder-contract-caller",
                &ink_e2e::alice(),
                constructor,
                0,
                None,
            )
            .await
            .expect("instantiate failed");
        let mut call = contract.call::<BuilderContractCaller>();

        // Check that the `total_supply` return `0`(default value).
        let value = client
            .call_dry_run(&ink_e2e::alice(), &call.total_supply(), 0, None)
            .await
            .return_value();
        assert_eq!(value, 0);
        // Mint tokens and transfer them to `to`.
        let to = AccountId::from([13; 32]);
        let amount = 100;
        let _ = client
            .call(
                &ink_e2e::alice(),
                &call.mint_and_transfer(to, amount),
                0,
                None,
            )
            .await
            .expect("calling `mint_and_transfer` failed");

        // The total supply should be equal to `amount`.
        let value = client
            .call_dry_run(&ink_e2e::alice(), &call.total_supply(), 0, None)
            .await
            .return_value();
        assert_eq!(value, amount);

        // The balance of the `to` should be equal to `amount`.
        let value = client
            .call_dry_run(&ink_e2e::alice(), &call.balance_of(to), 0, None)
            .await
            .return_value();
        assert_eq!(value, amount);

        Ok(())
    }
}
