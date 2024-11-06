use super::rlp::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn get_contract_storage_consumes_entire_buffer<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let mut constructor = RlpRef::new(false);
    let contract = client
        .instantiate("rlp", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let mut call_builder = contract.call_builder::<Rlp>();

    // when
    let result = client
        .call(
            &ink_e2e::alice(),
            &call_builder.set_value(true),
        )
        .submit()
        .await
        .expect("Calling `set_value` failed");

    Ok(())
}
