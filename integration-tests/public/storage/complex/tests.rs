use super::complex_structures::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn deployment_works(mut client: Client) -> E2EResult<()> {
    let mut constructor = ContractRef::new();

    let contract = client
        .instantiate("complex_storage_structures", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<Contract>    let get_balance = call_builder.get_balances_state();
    let get_balance_result = client
        .call(&ink_e2e::alice(), &get_balance)
        .submit()
        .await
        .expect("call failed");

    assert_eq!(get_balance_result.return_value(), 0);

    Ok(())
}