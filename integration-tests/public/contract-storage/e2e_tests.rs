use super::contract_storage::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn get_contract_storage_consumes_entire_buffer<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let mut constructor = ContractStorageRef::new();
    let contract = client
        .instantiate("contract-storage", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<ContractStorage>();

    // when
    let result = client
        .call(
            &ink_e2e::alice(),
            &call_builder.set_and_get_storage_all_data_consumed(),
        )
        .submit()
        .await
        .expect("Calling `insert_balance` failed")
        .return_value();

    assert!(result.is_ok());

    Ok(())
}

#[ink_e2e::test]
async fn get_contract_storage_fails_when_extra_data<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let mut constructor = ContractStorageRef::new();
    let contract = client
        .instantiate("contract-storage", &ink_e2e::bob(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<ContractStorage>();

    // when
    let result = client
        .call(
            &ink_e2e::bob(),
            &call_builder.set_and_get_storage_partial_data_consumed(),
        )
        .submit()
        .await;

    assert!(
        result.is_err(),
        "Expected the contract to revert when only partially consuming the buffer"
    );

    Ok(())
}

#[ink_e2e::test]
async fn take_contract_storage_consumes_entire_buffer<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let mut constructor = ContractStorageRef::new();
    let contract = client
        .instantiate("contract-storage", &ink_e2e::eve(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<ContractStorage>();

    // when
    let result = client
        .call(
            &ink_e2e::eve(),
            &call_builder.set_and_take_storage_all_data_consumed(),
        )
        .submit()
        .await
        .expect("Calling `insert_balance` failed")
        .return_value();

    assert!(result.is_ok());

    Ok(())
}

#[ink_e2e::test]
async fn take_contract_storage_fails_when_extra_data<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let mut constructor = ContractStorageRef::new();
    let contract = client
        .instantiate("contract-storage", &ink_e2e::ferdie(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<ContractStorage>();

    // when
    let result = client
        .call(
            &ink_e2e::ferdie(),
            &call_builder.set_and_take_storage_partial_data_consumed(),
        )
        .submit()
        .await;

    assert!(
        result.is_err(),
        "Expected the contract to revert when only partially consuming the buffer"
    );

    Ok(())
}
