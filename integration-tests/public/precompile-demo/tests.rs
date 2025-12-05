use super::precompile_demo::{PrecompileDemo, PrecompileDemoRef};
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn call_echo_works(mut client: Client) -> E2EResult<()> {
    // Given
    let mut constructor = PrecompileDemoRef::new();
    let contract = client
        .instantiate("precompile_demo", &ink_e2e::bob(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<PrecompileDemo>();

    // When
    let data = vec![0x1, 0x2, 0x3, 0x4];
    let expected = data.clone();
    let call_echo = call_builder.call_echo(data);
    let res = client
        .call(&ink_e2e::bob(), &call_echo)
        .submit()
        .await
        .expect("call_echo failed");

    // Then
    assert_eq!(res.return_value(), expected);

    Ok(())
}