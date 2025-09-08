use super::sol_cross_contract::*;
use ink_e2e::ContractsRegistry;
use ink_sandbox::{
    DefaultSandbox,
    Sandbox,
    api::prelude::*,
};

use ink::{
    Address,
    SolDecode,
    SolEncode,
    primitives::DepositLimit,
};
use ink_sandbox::frame_system::pallet_prelude::OriginFor;
use pallet_revive::ExecReturnValue;

const STORAGE_DEPOSIT_LIMIT: DepositLimit<u128> = DepositLimit::UnsafeOnlyForDryRun;

#[test]
fn call_sol_encoded_message() {
    let built_contracts = ::ink_e2e::build_root_and_contract_dependencies(vec![]);
    let contracts = ContractsRegistry::new(built_contracts);

    let mut sandbox = ink_e2e::DefaultSandbox::default();
    let caller = ink_e2e::alice();
    let origin =
        DefaultSandbox::convert_account_to_origin(DefaultSandbox::default_actor());

    sandbox
        .mint_into(&caller.public_key().0.into(), 1_000_000_000_000_000u128)
        .unwrap_or_else(|_| panic!("Failed to mint tokens"));

    sandbox.map_account(origin.clone()).expect("unable to map");

    // upload other contract (callee)
    let constructor = other_contract_sol::OtherContractRef::new(false);
    let params = constructor
        .endowment(0u32.into())
        .code_hash(ink::primitives::H256::zero())
        .salt_bytes(None)
        .params();
    let exec_input = params.exec_input();

    let code = contracts.load_code("other-contract-sol");
    let other_contract_addr = <DefaultSandbox as ContractAPI>::deploy_contract(
        &mut sandbox,
        code,
        0,
        exec_input.encode(),
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
    let constructor = SolCrossContractRef::new();
    let params = constructor
        .endowment(0u32.into())
        .code_hash(ink::primitives::H256::zero())
        .salt_bytes(None)
        .params();
    let exec_input = params.exec_input();

    let code = contracts.load_code("sol-cross-contract");
    let contract_addr = <DefaultSandbox as ContractAPI>::deploy_contract(
        &mut sandbox,
        code,
        0,
        exec_input.encode(),
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
        other_contract_addr,
        "get()",
        Vec::<u8>::new(),
        origin.clone(),
    );

    assert!(!value, "flip value should have been set to false");

    let input = other_contract_addr;

    // set value via cross contract call
    contracts.call(
        contract_addr,
        "call_contract_sol_encoding(address)",
        input,
        origin.clone(),
    );

    // get value
    let value: bool = contracts.call_with_return_value(
        other_contract_addr,
        "get()",
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
        contract_addr: Address,
        message: &str,
        args: Args,
        origin: OriginFor<<DefaultSandbox as Sandbox>::Runtime>,
    ) -> Ret
    where
        Args: for<'a> SolEncode<'a>,
        Ret: SolDecode,
    {
        let result = self.call(contract_addr, message, args, origin);
        Ret::decode(&result[..]).expect("decode failed")
    }

    fn call<Args>(
        &mut self,
        contract_addr: Address,
        message: &str,
        args: Args,
        origin: OriginFor<<DefaultSandbox as Sandbox>::Runtime>,
    ) -> Vec<u8>
    where
        Args: for<'a> SolEncode<'a>,
    {
        let mut data = keccak_selector(message.as_bytes());
        let mut encoded = args.encode();
        data.append(&mut encoded);

        let result = self.call_raw(contract_addr, data, origin);
        assert!(!result.did_revert(), "'{message}' failed {result:?}");
        result.data
    }

    fn call_raw(
        &mut self,
        contract_addr: Address,
        data: Vec<u8>,
        origin: OriginFor<<DefaultSandbox as Sandbox>::Runtime>,
    ) -> ExecReturnValue {
        let result = <DefaultSandbox as ContractAPI>::call_contract(
            &mut self.sandbox,
            contract_addr,
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
        Digest as _,
        digest::generic_array::GenericArray,
    };
    let mut hasher = sha3::Keccak256::new();
    hasher.update(input);
    hasher.finalize_into(<&mut GenericArray<u8, _>>::from(&mut output[..]));

    vec![output[0], output[1], output[2], output[3]]
}
