use super::cross_contract_calls::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn flip_and_get<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    // given
    let other_contract_code = client
        .upload("other-contract", &ink_e2e::alice())
        .submit()
        .await
        .expect("other_contract upload failed");

    let mut constructor = CrossContractCallsRef::new_v1(other_contract_code.code_hash);
    let contract = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("basic-contract-caller instantiate failed");
    let mut call_builder = contract.call_builder::<CrossContractCalls>();
    let call = call_builder.flip_and_get_v1();

    // when
    let result = client
        .call(&ink_e2e::alice(), &call)
        .submit()
        .await
        .expect("Calling `flip_and_get` failed")
        .return_value();

    assert!(!result);

    Ok(())
}

#[ink_e2e::test]
async fn instantiate_v2_with_insufficient_storage_deposit_limit<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let other_contract_code = client
        .upload("other-contract", &ink_e2e::alice())
        .submit()
        .await
        .expect("other_contract upload failed");

    const REF_TIME_LIMIT: u64 = 500_000_000;
    const PROOF_SIZE_LIMIT: u64 = 100_000;
    const STORAGE_DEPOSIT_LIMIT: u128 = 100_000_000_000;

    let mut constructor = CrossContractCallsRef::new_v2_with_limits(
        other_contract_code.code_hash,
        REF_TIME_LIMIT,
        PROOF_SIZE_LIMIT,
        STORAGE_DEPOSIT_LIMIT,
    );
    let contract = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await;

    let Err(ink_e2e::Error::InstantiateDryRun(err)) = contract else {
        panic!("instantiate should have failed at the dry run");
    };

    // insufficient storage deposit limit
    assert!(
        err.error
            .to_string()
            .contains("StorageDepositLimitExhausted"),
        "should have failed with StorageDepositLimitExhausted"
    );

    Ok(())
}

#[ink_e2e::test]
async fn instantiate_v2_with_sufficient_limits<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let other_contract_code = client
        .upload("other-contract", &ink_e2e::alice())
        .submit()
        .await
        .expect("other_contract upload failed");

    const REF_TIME_LIMIT: u64 = 500_000_000;
    const PROOF_SIZE_LIMIT: u64 = 100_000;
    const STORAGE_DEPOSIT_LIMIT: u128 = 500_000_000_000;

    let mut constructor = CrossContractCallsRef::new_v2_with_limits(
        other_contract_code.code_hash,
        REF_TIME_LIMIT,
        PROOF_SIZE_LIMIT,
        STORAGE_DEPOSIT_LIMIT,
    );
    let contract = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await;

    assert!(contract.is_ok(), "{}", contract.err().unwrap());

    Ok(())
}

#[ink_e2e::test]
async fn instantiate_v2_no_limits<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let other_contract_code = client
        .upload("other-contract", &ink_e2e::alice())
        .submit()
        .await
        .expect("other_contract upload failed");

    let mut constructor =
        CrossContractCallsRef::new_v2_no_limits(other_contract_code.code_hash);
    let contract = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await;

    assert!(contract.is_ok(), "{}", contract.err().unwrap());

    Ok(())
}

#[ink_e2e::test]
async fn flip_and_get_v2<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    // given
    let other_contract_code = client
        .upload("other-contract", &ink_e2e::alice())
        .submit()
        .await
        .expect("other_contract upload failed");

    let mut constructor = CrossContractCallsRef::new_v1(other_contract_code.code_hash);
    let contract = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("cross-contract-calls instantiate failed");
    let mut call_builder = contract.call_builder::<CrossContractCalls>();

    const REF_TIME_LIMIT: u64 = 500_000_000;
    const PROOF_SIZE_LIMIT: u64 = 100_000;
    const STORAGE_DEPOSIT_LIMIT: u128 = 1_000_000_000;

    // when
    let call = call_builder.flip_and_get_invoke_v2_with_limits(
        REF_TIME_LIMIT,
        PROOF_SIZE_LIMIT,
        STORAGE_DEPOSIT_LIMIT,
    );
    let result = client
        .call(&ink_e2e::alice(), &call)
        .submit()
        .await
        .expect("Calling `flip_and_get` failed")
        .return_value();

    assert!(!result);

    let call = call_builder.flip_and_get_invoke_v2_no_weight_limit();
    let result = client
        .call(&ink_e2e::alice(), &call)
        .submit()
        .await
        .expect("Calling `flip_and_get` failed")
        .return_value();

    assert!(result);

    Ok(())
}
