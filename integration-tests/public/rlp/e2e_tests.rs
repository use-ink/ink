use super::rlp::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test(backend(runtime_only))]
async fn call_rlp_encoded_message<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    let caller = ink_e2e::alice();
    // given
    let mut constructor = RlpRef::new(false);
    // todo: instantiate the contract via the sandbox instance...
    let contract = client
        .instantiate("rlp", &caller, &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    
    let set_value_selector = Vec::<u8>::new(); // todo: calculate and append selector
    let mut set_value_data = Vec::new();
    ink::rlp::Encodable::encode(&true, &mut set_value_data);
    
    let mut sandbox = ink_e2e::DefaultSandbox::default();
    let account_id = (*<ink::primitives::AccountId as AsRef<[u8; 32]>>::as_ref(&contract.account_id)).into();

    <ink_e2e::DefaultSandbox as ink_sandbox::api::contracts_api::ContractAPI>
        ::call_contract(
            &mut sandbox,
            account_id,
            0,
            set_value_data,
            caller.public_key().0.into(),
            <ink_e2e::DefaultSandbox as ink_sandbox::Sandbox>::default_gas_limit(),
            None,
            pallet_contracts::Determinism::Enforced,
        )
        .result
        .expect("sandbox call contract failed");
    
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
