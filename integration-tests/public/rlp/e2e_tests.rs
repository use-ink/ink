use super::rlp::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test(backend(runtime_only))]
async fn call_rlp_encoded_message<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // given
    let mut constructor = RlpRef::new(false);
    // the constructor is still SCALE encoded so use the E2E client still
    let contract = client
        .instantiate("rlp", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    
    let set_value_selector = Vec::<u8>::new(); // todo: calculate selector
    let mut buf = Vec::new();
    let set_value_data = ink::rlp::Encodable::encode(&true, &mut buf);
    
    let sandbox = ink_e2e::DefaultSandbox::default();
    
    // let mut call_builder = contract.call_builder::<Rlp>();
    // 
    // // when
    // let result = client
    //     .call(&ink_e2e::alice(), &call_builder.set_value(true))
    //     .submit()
    //     .await
    //     .expect("Calling `set_value` failed");

    Ok(())
}
