use super::rlp::*;
use ink_e2e::{
    ContractsRegistry,
    Keypair,
};
use ink_sandbox::{api::prelude::*, AccountId32, DefaultSandbox, Sandbox};
use pallet_revive::ExecReturnValue;
use ink::H160;
use ink::primitives::{AccountId, DepositLimit};
use ink_e2e::subxt::tx::Signer;
const STORAGE_DEPOSIT_LIMIT: DepositLimit<u128> = DepositLimit::Unchecked;

#[test]
fn call_rlp_encoded_message() {
    let built_contracts = ::ink_e2e::build_root_and_contract_dependencies();
    let contracts = ContractsRegistry::new(built_contracts);

    let mut sandbox = ink_e2e::DefaultSandbox::default();
    let caller = ink_e2e::alice();

    sandbox
        .mint_into(
            &caller.public_key().0.into(),
            1_000_000_000_000_000u128.into(),
        )
        .unwrap_or_else(|_| panic!("Failed to mint tokens"));

    let origin = DefaultSandbox::convert_account_to_origin(AccountId32::from(caller.public_key().0));
    sandbox.map_account(origin.clone()).expect("unable to map");

    // given
    let constructor = RlpRef::new(false);
    let params = constructor
        .endowment(0u32.into())
        .code_hash(ink::primitives::H256::zero())
        .salt_bytes(None)
        .params();
    let exec_input = params.exec_input();

    // TODO: could potentially simplify if helpers are exposed.

    let code = contracts.load_code("rlp");
    let contract_addr = <DefaultSandbox as ink_sandbox::api::revive_api::ContractAPI>
        ::deploy_contract(
            &mut sandbox,
            code,
            0,
            ink::scale::Encode::encode(&exec_input),
            // salt
            None,
            origin,
            <DefaultSandbox as Sandbox>::default_gas_limit(),
            STORAGE_DEPOSIT_LIMIT
        )
        .result
        .expect("sandbox deploy contract failed").addr;

    let mut contract = ContractSandbox {
        sandbox,
        contract_addr: contract_addr,
    };

    // set value
    contract.call("set_value", true, caller.clone());

    // get value
    let value: bool =
        contract.call_with_return_value("get_value", Vec::<u8>::new(), caller);

    assert!(value, "value should have been set to true");
}

struct ContractSandbox {
    sandbox: ink_e2e::DefaultSandbox,
    contract_addr: H160,
}

impl ContractSandbox {
    fn call_with_return_value<Args, Ret>(
        &mut self,
        message: &str,
        args: Args,
        caller: Keypair,
    ) -> Ret
    where
        Args: ink::rlp::Encodable,
        Ret: ink::rlp::Decodable,
    {
        let result = self.call(message, args, caller);
        ink::rlp::Decodable::decode(&mut &result[..]).expect("decode failed")
    }

    fn call<Args>(&mut self, message: &str, args: Args, caller: Keypair) -> Vec<u8>
    where
        Args: ink::rlp::Encodable,
    {
        let mut data = keccak_selector(message.as_bytes());
        let mut args_buf = Vec::new();
        ink::rlp::Encodable::encode(&args, &mut args_buf);
        data.append(&mut args_buf);

        let result = self.call_raw(data, caller);
        assert!(!result.did_revert(), "'{message}' failed {:?}", result);
        result.data
    }


    fn call_raw(&mut self, data: Vec<u8>, caller: Keypair) -> ExecReturnValue {
        let origin = DefaultSandbox::convert_account_to_origin(AccountId32::from(caller.public_key().0));
        let result =
            <DefaultSandbox as ink_sandbox::api::revive_api::ContractAPI>
                ::call_contract(
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
