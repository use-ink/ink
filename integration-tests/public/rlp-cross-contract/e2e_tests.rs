use super::rlp_cross_contract::*;
use ink_e2e::ContractsRegistry;
use ink_sandbox::{
    api::prelude::*,
    DefaultSandbox,
    Sandbox,
};

use ink::{
    primitives::DepositLimit,
    H160,
};
use ink_sandbox::frame_system::pallet_prelude::OriginFor;
use pallet_revive::ExecReturnValue;

const STORAGE_DEPOSIT_LIMIT: DepositLimit<u128> = DepositLimit::Unchecked;

#[test]
fn call_rlp_encoded_message() {
    let built_contracts = ::ink_e2e::build_root_and_contract_dependencies();
    let contracts = ContractsRegistry::new(built_contracts);

    let mut sandbox = ink_e2e::DefaultSandbox::default();
    let caller = ink_e2e::alice();
    let origin =
        DefaultSandbox::convert_account_to_origin(DefaultSandbox::default_actor());

    sandbox
        .mint_into(
            &caller.public_key().0.into(),
            1_000_000_000_000_000u128.into(),
        )
        .unwrap_or_else(|_| panic!("Failed to mint tokens"));

    sandbox.map_account(origin.clone()).expect("unable to map");

    // upload other contract (callee)
    let constructor = other_contract_rlp::OtherContractRef::new(false);
    let params = constructor
        .endowment(0u32.into())
        .code_hash(ink::primitives::H256::zero())
        .salt_bytes(None)
        .params();
    let exec_input = params.exec_input();

    let code = contracts.load_code("other-contract-rlp");
    let other_contract_addr = <DefaultSandbox as ContractAPI>::deploy_contract(
        &mut sandbox,
        code,
        0,
        ink::scale::Encode::encode(&exec_input),
        // salt
        None,
        origin.clone(),
        <DefaultSandbox as Sandbox>::default_gas_limit(),
        STORAGE_DEPOSIT_LIMIT,
    )
    .result
    .expect("sandbox deploy contract failed")
    .addr;

    // upload main contract (caller)
    let constructor = RlpCrossContractRef::new();
    let params = constructor
        .endowment(0u32.into())
        .code_hash(ink::primitives::H256::zero())
        .salt_bytes(None)
        .params();
    let exec_input = params.exec_input();

    let code = contracts.load_code("rlp-cross-contract");
    let contract_addr = <DefaultSandbox as ContractAPI>::deploy_contract(
        &mut sandbox,
        code,
        0,
        ink::scale::Encode::encode(&exec_input),
        // salt
        // TODO (@peterwht): figure out why no salt is causing `DuplicateContract`
        Some([1u8; 32]),
        origin.clone(),
        <DefaultSandbox as Sandbox>::default_gas_limit(),
        STORAGE_DEPOSIT_LIMIT,
    )
    .result
    .expect("sandbox deploy contract failed")
    .addr;

    let mut contracts = ContractSandbox { sandbox };

    // get value
    let value: bool = contracts.call_with_return_value(
        other_contract_addr.clone(),
        "get",
        Vec::<u8>::new(),
        origin.clone(),
    );

    assert!(!value, "flip value should have been set to false");

    let input: [u8; 20] = other_contract_addr.clone().into();

    // set value via cross contract call
    contracts.call(contract_addr, "call_contract_rlp", input, origin.clone());

    // get value
    let value: bool = contracts.call_with_return_value(
        other_contract_addr,
        "get",
        Vec::<u8>::new(),
        origin,
    );

    assert!(value, "value should have been set to true");
}

struct ContractSandbox {
    sandbox: ink_e2e::DefaultSandbox,
}

impl ContractSandbox {
    fn call_with_return_value<Args, Ret>(
        &mut self,
        contract_addr: H160,
        message: &str,
        args: Args,
        origin: OriginFor<<DefaultSandbox as Sandbox>::Runtime>,
    ) -> Ret
    where
        Args: ink::rlp::Encodable,
        Ret: ink::rlp::Decodable,
    {
        let result = self.call(contract_addr, message, args, origin);
        ink::rlp::Decodable::decode(&mut &result[..]).expect("decode failed")
    }

    fn call<Args>(
        &mut self,
        contract_addr: H160,
        message: &str,
        args: Args,
        origin: OriginFor<<DefaultSandbox as Sandbox>::Runtime>,
    ) -> Vec<u8>
    where
        Args: ink::rlp::Encodable,
    {
        let mut data = keccak_selector(message.as_bytes());
        let mut args_buf = Vec::new();
        ink::rlp::Encodable::encode(&args, &mut args_buf);
        data.append(&mut args_buf);

        let result = self.call_raw(contract_addr, data, origin);
        assert!(!result.did_revert(), "'{message}' failed {:?}", result);
        result.data
    }

    fn call_raw(
        &mut self,
        contract_addr: H160,
        data: Vec<u8>,
        origin: OriginFor<<DefaultSandbox as Sandbox>::Runtime>,
    ) -> ExecReturnValue {
        let result = <DefaultSandbox as ContractAPI>::call_contract(
            &mut self.sandbox,
            contract_addr.clone(),
            0,
            data,
            origin,
            <DefaultSandbox as Sandbox>::default_gas_limit(),
            STORAGE_DEPOSIT_LIMIT,
        );

        result.result.expect("sandbox call contract failed")
    }
}

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
