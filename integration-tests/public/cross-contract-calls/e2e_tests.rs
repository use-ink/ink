use super::cross_contract_calls::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn instantiate_with_insufficient_storage_deposit_limit<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let other_contract_code = client
        .upload("other-contract", &ink_e2e::alice())
        .submit()
        .await
        .expect("other_contract upload failed");

    const REF_TIME_LIMIT: u64 = 500;
    const PROOF_SIZE_LIMIT: u64 = 100_000_000_000;
    let storage_deposit_limit = ink::U256::from(100_000_000_000_000u64);

    let mut constructor = CrossContractCallsRef::new_with_limits(
        other_contract_code.code_hash,
        REF_TIME_LIMIT,
        PROOF_SIZE_LIMIT,
        storage_deposit_limit,
    );
    let call_result = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .dry_run()
        .await?;

    assert!(call_result.did_revert());
    let err_msg = String::from_utf8_lossy(call_result.return_data());
    assert!(err_msg.contains(
        "Cross-contract instantiation failed with ReturnError(OutOfResources)"
    ));

    Ok(())
}

#[ink_e2e::test]
async fn instantiate_with_sufficient_limits<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let other_contract_code = client
        .upload("other-contract", &ink_e2e::alice())
        .submit()
        .await
        .expect("other_contract upload failed");

    const REF_TIME_LIMIT: u64 = 500_000_000_000_000;
    const PROOF_SIZE_LIMIT: u64 = 100_000_000_000;
    // todo remove the last group of `000` to get an `OutOfGas` error in
    // `pallet-revive`. but they should throw an error about `StorageLimitExhausted`.
    let storage_deposit_limit = ink::U256::from(100_000_000_000_000u64);

    let mut constructor = CrossContractCallsRef::new_with_limits(
        other_contract_code.code_hash,
        REF_TIME_LIMIT,
        PROOF_SIZE_LIMIT,
        storage_deposit_limit,
    );
    let contract = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await;

    assert!(contract.is_ok(), "{}", contract.err().unwrap());

    Ok(())
}

#[ink_e2e::test]
async fn instantiate_no_limits<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    // given
    let other_contract_code = client
        .upload("other-contract", &ink_e2e::alice())
        .submit()
        .await
        .expect("other_contract upload failed");

    let mut constructor =
        CrossContractCallsRef::new_no_limits(other_contract_code.code_hash);
    let contract = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await;

    assert!(contract.is_ok(), "{}", contract.err().unwrap());

    Ok(())
}

#[ink_e2e::test]
async fn flip_and_get<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    // given
    let other_contract_code = client
        .upload("other-contract", &ink_e2e::alice())
        .submit()
        .await
        .expect("other_contract upload failed");

    let mut constructor =
        CrossContractCallsRef::new_no_limits(other_contract_code.code_hash);
    let contract = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("cross-contract-calls instantiate failed");
    let mut call_builder = contract.call_builder::<CrossContractCalls>();

    const REF_TIME_LIMIT: u64 = 500_000_000;
    const PROOF_SIZE_LIMIT: u64 = 100_000;
    let storage_deposit_limit = ink::U256::from(1_000_000_000);

    // when
    let call = call_builder.flip_and_get_invoke_with_limits(
        REF_TIME_LIMIT,
        PROOF_SIZE_LIMIT,
        storage_deposit_limit,
    );
    let result = client
        .call(&ink_e2e::alice(), &call)
        .submit()
        .await
        .expect("Calling `flip_and_get_invoke_with_limits` failed")
        .return_value();

    assert!(!result);

    let call = call_builder.flip_and_get_invoke_no_weight_limit();
    let result = client
        .call(&ink_e2e::alice(), &call)
        .submit()
        .await
        .expect("Calling `flip_and_get_invoke_no_weight_limit` failed")
        .return_value();

    assert!(result);

    Ok(())
}
