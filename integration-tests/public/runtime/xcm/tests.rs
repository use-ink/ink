#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::contract_xcm::{ContractXcm, ContractXcmRef};
    use ink::primitives::AccountId;
    use ink_e2e::{
        ChainBackend,
        ContractsBackend,
    };

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    /// Tests that `xcm_execute` correctly transfers funds locally via XCM instructions.
    #[ink_e2e::test]
    async fn xcm_execute_works(mut client: Client) -> E2EResult<()> {
        // Given: Instantiate contract with an initial endowment.
        let mut constructor = ContractXcmRef::new();
        let contract = client
            .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
            .value(100_000_000_000)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder = contract.call_builder::<ContractXcm>();

        let receiver = AccountId::from(ink_e2e::bob().public_key().0);

        let contract_balance_before = client
            .free_balance(contract.account_id)
            .await
            .expect("Failed to get account balance");
        let receiver_balance_before = client
            .free_balance(receiver)
            .await
            .expect("Failed to get account balance");

        // When: Execute XCM transfer of 100_000_000 units to receiver.
        let amount = 100_000_000;
        let transfer_message = call_builder.transfer_through_xcm(receiver, amount);
        let call_res = client
            .call(&ink_e2e::alice(), &transfer_message)
            .submit()
            .await
            .expect("call failed");
        assert!(call_res.return_value().is_ok());

        // Then: Verify balances updated correctly.
        let contract_balance_after = client
            .free_balance(contract.account_id)
            .await
            .expect("Failed to get account balance");
        let receiver_balance_after = client
            .free_balance(receiver)
            .await
            .expect("Failed to get account balance");

        assert_eq!(contract_balance_after, contract_balance_before - amount);
        assert_eq!(receiver_balance_after, receiver_balance_before + amount);

        Ok(())
    }

    /// Tests that `xcm_execute` fails gracefully when funds are insufficient.
    #[ink_e2e::test]
    async fn xcm_execute_failure_detection_works(
        mut client: Client,
    ) -> E2EResult<()> {
        // Sleep to avoid nonce collision with other tests using Alice.
        std::thread::sleep(std::time::Duration::from_secs(10));

        // Given: Instantiate contract.
        let mut constructor = ContractXcmRef::new();
        let contract = client
            .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder = contract.call_builder::<ContractXcm>();

        // When: Try to transfer `u128::MAX` (impossible amount).
        let receiver = AccountId::from(ink_e2e::bob().public_key().0);
        let amount = u128::MAX;
        let transfer_message = call_builder.transfer_through_xcm(receiver, amount);

        // Then: The call should return an error indicating execution failure.
        let call_res = client
            .call(&ink_e2e::alice(), &transfer_message)
            .submit()
            .await;
        assert!(call_res.is_err());

        // Verify the specific error message.
        let expected = "revert: XCM execute failed: message may be invalid or execution constraints not satisfied";
        assert!(format!("{:?}", call_res).contains(expected));

        Ok(())
    }

    /// Tests that `xcm_send` successfully dispatches a message to the Relay Chain.
    /// Note: This tests the *dispatch*, not the successful execution on the remote chain.
    #[ink_e2e::test]
    async fn xcm_send_works(mut client: Client) -> E2EResult<()> {
        // Sleep to avoid nonce collision.
        std::thread::sleep(std::time::Duration::from_secs(30));

        // Given: Instantiate contract with funds.
        let mut constructor = ContractXcmRef::new();
        let contract = client
            .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
            .value(100_000_000_000)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder = contract.call_builder::<ContractXcm>();

        let contract_balance_before = client
            .free_balance(contract.account_id)
            .await
            .expect("Failed to get account balance");

        // When: Send funds via XCM to the Relay Chain.
        let amount = 100_000_000;
        let transfer_message = call_builder.send_funds(amount, amount / 2);
        let call_res = client
            .call(&ink_e2e::alice(), &transfer_message)
            .submit()
            .await
            .expect("call failed");
        assert!(call_res.return_value().is_ok());

        // Then: Contract balance should decrease (amount sent + execution fees).
        let contract_balance_after = client
            .free_balance(contract.account_id)
            .await
            .expect("Failed to get account balance");

        assert!(
            contract_balance_after <= contract_balance_before - amount - (amount / 2)
        );

        Ok(())
    }
}