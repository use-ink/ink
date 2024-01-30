use super::basic_contract_caller::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn flip_and_get<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let other_contract_code = client
        .upload("other-contract", &ink_e2e::alice())
        .submit()
        .await
        .expect("other_contract upload failed");

    let mut constructor = BasicContractCallerRef::new(other_contract_code.code_hash);
    let contract = client
        .instantiate("basic-contract-caller", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("basic-contract-caller instantiate failed");
    let call_builder = contract.call_builder::<BasicContractCaller>();
    let call = call_builder.flip_and_get();

    // when
    let result = client
        .call(
            &ink_e2e::alice(),
            &call,
        )
        .submit()
        .await
        .expect("Calling `flip_and_get` failed")
        .return_value();

    assert!(result);

    Ok(())
}
