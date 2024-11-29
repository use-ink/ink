use super::rlp::*;
use ink_e2e::{
    ContractsRegistry
};
use ink_sandbox::api::balance_api::BalanceAPI;

#[test]
fn call_rlp_encoded_message() {

    let built_contracts = ::ink_e2e::build_root_and_contract_dependencies();
    let contracts = ContractsRegistry::new(built_contracts);

    let mut sandbox = ink_e2e::DefaultSandbox::default();
    let caller = ink_e2e::alice();

    sandbox
        .mint_into(&caller.public_key().0.into(), 1_000_000_000_000_000u128.into())
        .unwrap_or_else(|_| panic!("Failed to mint tokens"));

    // given
    let constructor = RlpRef::new(false);
    let params = constructor
        .endowment(0u32.into())
        .code_hash(ink::primitives::Clear::CLEAR_HASH)
        .salt_bytes(Vec::new())
        .params();
    let exec_input = params.exec_input();

    let code = contracts.load_code("rlp");
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

    fn keccak_selector(input: &[u8]) -> Vec<u8> {
        let mut output = [0; 32];
        use sha3::{
            digest::generic_array::GenericArray,
            Digest as _,
        };
        let mut hasher = sha3::Keccak256::new();
        hasher.update(input);
        hasher.finalize_into(<&mut GenericArray<u8, _>>::from(&mut output[..]));
        vec![output[0], output[1], output[2], output[3]]
    }

    // set value
    let mut set_value_data = keccak_selector(b"Rlp::set_value");
    let mut value_buf = Vec::new();
    ink::rlp::Encodable::encode(&true, &mut value_buf);
    set_value_data.append(&mut value_buf);

    let result =
        <ink_e2e::DefaultSandbox as ink_sandbox::api::contracts_api::ContractAPI>
            ::call_contract(
                &mut sandbox,
                contract_account_id.clone(),
                0,
                set_value_data,
                caller.public_key().0.into(),
                <ink_e2e::DefaultSandbox as ink_sandbox::Sandbox>::default_gas_limit(),
                None,
                pallet_contracts::Determinism::Enforced,
            )
            .result
            .expect("sandbox call contract failed");
    println!("result: {:?}", result);

    // get value
    let mut get_value_data = keccak_selector(b"Rlp::get_value");
    let result =
        <ink_e2e::DefaultSandbox as ink_sandbox::api::contracts_api::ContractAPI>
            ::call_contract(
                &mut sandbox,
                contract_account_id,
                0,
                get_value_data,
                caller.public_key().0.into(),
                <ink_e2e::DefaultSandbox as ink_sandbox::Sandbox>::default_gas_limit(),
                None,
                pallet_contracts::Determinism::Enforced,
            )
            .result
            .expect("sandbox call contract failed");
    println!("result: {:?}", result);
}
