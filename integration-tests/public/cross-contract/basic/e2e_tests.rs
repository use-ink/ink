use super::cross_contract_calls::*;
use ink_e2e::ContractsBackend;
use other_contract::OtherContractRef;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn instantiate_no_limits(mut client: Client) -> E2EResult<()> {
    // given
    let mut other_constructor = OtherContractRef::new(true);
    let other_contract = client
        .instantiate("other-contract", &ink_e2e::alice(), &mut other_constructor)
        .submit()
        .await
        .expect("other-contract instantiate failed");

    // when
    let mut constructor = CrossContractCallsRef::new(other_contract.addr);
    let contract = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await;

    // then
    assert!(contract.is_ok(), "{}", contract.err().unwrap());

    Ok(())
}

#[ink_e2e::test]
async fn flip_and_get(mut client: Client) -> E2EResult<()> {
    // given
    let mut other_constructor = OtherContractRef::new(true);
    let other_contract = client
        .instantiate("other-contract", &ink_e2e::alice(), &mut other_constructor)
        .submit()
        .await
        .expect("other-contract instantiate failed");

    let mut constructor = CrossContractCallsRef::new(other_contract.addr);
    let contract = client
        .instantiate("cross-contract-calls", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("cross-contract-calls instantiate failed");
    let mut call_builder = contract.call_builder::<CrossContractCalls>();

    // when
    let call = call_builder.flip_and_get();
    let result = client
        .call(&ink_e2e::alice(), &call)
        .submit()
        .await
        .expect("Calling `flip_and_get` failed")
        .return_value();

    // then
    assert!(!result);

    // when
    let call = call_builder.flip_and_get();
    let result = client
        .call(&ink_e2e::alice(), &call)
        .submit()
        .await
        .expect("Calling `flip_and_get` failed")
        .return_value();

    // then
    assert!(result);

    Ok(())
}
