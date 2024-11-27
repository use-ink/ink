use super::rlp::*;
use ink_e2e::{
    ContractsBackend
};

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test(backend(runtime_only))]
async fn call_rlp_encoded_message<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    let mut sandbox = ink_e2e::DefaultSandbox::default();

    let caller = ink_e2e::alice();
    // given
    let mut constructor = RlpRef::new(false);
    let exec_input = constructor
        .endowment(0u32.into())
        .code_hash(ink::primitives::Clear::CLEAR_HASH)
        .salt_bytes(Vec::new())
        .params()
        .exec_input();

    let code = client.contracts().load_code("rlp");
    let contract_account_id = <ink_e2e::DefaultSandbox as ink_sandbox::api::contracts_api::ContractAPI>
        ::deploy_contract(
            &mut sandbox,
            code,
            0,
            ink::scale::Encode::encode(&exec_input),
            vec![0u8],
            caller.public_key().0.into(),
            <ink_e2e::DefaultSandbox as ink_sandbox::Sandbox>::default_gas_limit(),
            None,
        )
        .result
        .expect("sandbox deploy contract failed")
        .account_id;
    
    let account_id = (*<ink::primitives::AccountId as AsRef<[u8; 32]>>::as_ref(&contract_account_id)).into();

    let set_value_selector = Vec::<u8>::new(); // todo: calculate and append selector
    let mut set_value_data = Vec::new();
    ink::rlp::Encodable::encode(&true, &mut set_value_data);

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

    Ok(())
}
