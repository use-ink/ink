use super::contract_storage::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn get_contract_storage_consumes_entire_buffer<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let constructor = ContractStorageRef::new();
    let contract = client
        .instantiate("contract-storage", &ink_e2e::alice(), constructor, 0, None)
        .await
        .expect("instantiate failed");
    let call = contract.call::<ContractStorage>();

    // when
    let result = client
        .call(
            &ink_e2e::alice(),
            &call.set_and_get_storage_all_data_consumed(),
            0,
            None,
        )
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
    let constructor = ContractStorageRef::new();
    let contract = client
        .instantiate("contract-storage", &ink_e2e::alice(), constructor, 0, None)
        .await
        .expect("instantiate failed");
    let call = contract.call::<ContractStorage>();

    // when
    let result = client
        .call(
            &ink_e2e::alice(),
            &call.set_and_get_storage_partial_data_consumed(),
            0,
            None,
        )
        .await;

    assert!(
        result.is_err(),
        "Expected the contract to revert when only partially consuming the buffer"
    );

    Ok(())
}
