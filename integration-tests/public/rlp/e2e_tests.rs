use super::rlp::*;
use ink::{
    primitives::DepositLimit,
    H160,
};
use ink_e2e::ContractsRegistry;
use ink_sandbox::{
    api::prelude::*,
    frame_system::pallet_prelude::OriginFor,
    DefaultSandbox,
    Sandbox,
};
use pallet_revive::ExecReturnValue;

const STORAGE_DEPOSIT_LIMIT: DepositLimit<u128> = DepositLimit::Unchecked;

#[test]
fn call_rlp_encoded_message() {
    let built_contracts = ::ink_e2e::build_root_and_contract_dependencies();
    let contracts = ContractsRegistry::new(built_contracts);

    let mut sandbox = DefaultSandbox::default();
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

    let constructor = RlpRef::new(false);
    let params = constructor
        .endowment(0u32.into())
        .code_hash(ink::primitives::H256::zero())
        .salt_bytes(None)
        .params();
    let exec_input = params.exec_input();

    let code = contracts.load_code("rlp");
    let contract_addr = <DefaultSandbox as ContractAPI>::deploy_contract(
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

    let mut contract = ContractSandbox {
        sandbox,
        contract_addr,
    };

    // set value
    contract.call("set_value", true, origin.clone());

    // get value
    let value: bool =
        contract.call_with_return_value("get_value", Vec::<u8>::new(), origin);

    assert!(value, "value should have been set to true");
}

struct ContractSandbox {
    sandbox: DefaultSandbox,
    contract_addr: H160,
}

impl ContractSandbox {
    fn call_with_return_value<Args, Ret>(
        &mut self,
        message: &str,
        args: Args,
        origin: OriginFor<<DefaultSandbox as Sandbox>::Runtime>,
    ) -> Ret
    where
        Args: ink::rlp::Encodable,
        Ret: ink::rlp::Decodable,
    {
        let result = self.call(message, args, origin);
        ink::rlp::Decodable::decode(&mut &result[..]).expect("decode failed")
    }

    fn call<Args>(
        &mut self,
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

        let result = self.call_raw(data, origin);
        assert!(!result.did_revert(), "'{message}' failed {:?}", result);
        result.data
    }

    fn call_raw(
        &mut self,
        data: Vec<u8>,
        origin: OriginFor<<DefaultSandbox as Sandbox>::Runtime>,
    ) -> ExecReturnValue {
        let result = <DefaultSandbox as ContractAPI>::call_contract(
            &mut self.sandbox,
            self.contract_addr.clone(),
            0,
            data,
            origin,
            <DefaultSandbox as Sandbox>::default_gas_limit(),
            STORAGE_DEPOSIT_LIMIT,
        )
        .result
        .expect("sandbox call contract failed");
        result
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
