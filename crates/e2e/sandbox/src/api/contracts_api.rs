use crate::{
    AccountIdFor,
    ContractExecResultFor,
    ContractInstantiateResultFor,
    EventRecordOf,
    Sandbox,
};
use frame_support::{
    traits::fungible::Inspect,
    weights::Weight,
};
use frame_system::Config as SysConfig;
use pallet_contracts::{
    Code,
    CodeUploadResult,
    CollectEvents,
    ContractInstantiateResult,
    DebugInfo,
    Determinism,
};
use scale::Decode as _;
use std::ops::Not;

type BalanceOf<R> =
    <<R as pallet_contracts::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;

/// Contract API used to interact with the contracts pallet.
pub trait ContractAPI {
    /// The runtime contract config.
    type T: pallet_contracts::Config;

    /// Interface for `bare_instantiate` contract call with a simultaneous upload.
    ///
    /// # Arguments
    ///
    /// * `contract_bytes` - The contract code.
    /// * `value` - The number of tokens to be transferred to the contract.
    /// * `data` - The input data to be passed to the contract (including constructor
    ///   name).
    /// * `salt` - The salt to be used for contract address derivation.
    /// * `origin` - The sender of the contract call.
    /// * `gas_limit` - The gas limit for the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn deploy_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        salt: Vec<u8>,
        origin: AccountIdFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: Option<BalanceOf<Self::T>>,
    ) -> ContractInstantiateResult<
        AccountIdFor<Self::T>,
        BalanceOf<Self::T>,
        EventRecordOf<Self::T>,
    >;

    /// Interface for `bare_instantiate` contract call for a previously uploaded contract.
    ///
    /// # Arguments
    ///
    /// * `code_hash` - The code hash of the contract to instantiate.
    /// * `value` - The number of tokens to be transferred to the contract.
    /// * `data` - The input data to be passed to the contract (including constructor
    ///   name).
    /// * `salt` - The salt to be used for contract address derivation.
    /// * `origin` - The sender of the contract call.
    /// * `gas_limit` - The gas limit for the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn instantiate_contract(
        &mut self,
        code_hash: Vec<u8>,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        salt: Vec<u8>,
        origin: AccountIdFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: Option<BalanceOf<Self::T>>,
    ) -> ContractInstantiateResult<
        AccountIdFor<Self::T>,
        BalanceOf<Self::T>,
        EventRecordOf<Self::T>,
    >;

    /// Interface for `bare_upload_code` contract call.
    ///
    /// # Arguments
    ///
    /// * `contract_bytes` - The contract code.
    /// * `origin` - The sender of the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    fn upload_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        origin: AccountIdFor<Self::T>,
        storage_deposit_limit: Option<BalanceOf<Self::T>>,
        determinism: Determinism,
    ) -> CodeUploadResult<<Self::T as frame_system::Config>::Hash, BalanceOf<Self::T>>;

    /// Interface for `bare_call` contract call.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the contract to be called.
    /// * `value` - The number of tokens to be transferred to the contract.
    /// * `data` - The input data to be passed to the contract (including message name).
    /// * `origin` - The sender of the contract call.
    /// * `gas_limit` - The gas limit for the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn call_contract(
        &mut self,
        address: AccountIdFor<Self::T>,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        origin: AccountIdFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: Option<BalanceOf<Self::T>>,
        determinism: Determinism,
    ) -> ContractExecResultFor<Self::T>;
}

impl<T> ContractAPI for T
where
    T: Sandbox,
    T::Runtime: pallet_contracts::Config,
{
    type T = T::Runtime;

    fn deploy_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        salt: Vec<u8>,
        origin: AccountIdFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: Option<BalanceOf<Self::T>>,
    ) -> ContractInstantiateResultFor<Self::T> {
        self.execute_with(|| {
            pallet_contracts::Pallet::<Self::T>::bare_instantiate(
                origin,
                value,
                gas_limit,
                storage_deposit_limit,
                Code::Upload(contract_bytes),
                data,
                salt,
                DebugInfo::UnsafeDebug,
                CollectEvents::UnsafeCollect,
            )
        })
    }

    fn instantiate_contract(
        &mut self,
        code_hash: Vec<u8>,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        salt: Vec<u8>,
        origin: AccountIdFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: Option<BalanceOf<Self::T>>,
    ) -> ContractInstantiateResult<
        AccountIdFor<Self::T>,
        BalanceOf<Self::T>,
        EventRecordOf<Self::T>,
    > {
        let mut code_hash = &code_hash[..];
        self.execute_with(|| {
            pallet_contracts::Pallet::<Self::T>::bare_instantiate(
                origin,
                value,
                gas_limit,
                storage_deposit_limit,
                Code::Existing(
                    <Self::T as SysConfig>::Hash::decode(&mut code_hash)
                        .expect("Invalid code hash"),
                ),
                data,
                salt,
                DebugInfo::UnsafeDebug,
                CollectEvents::UnsafeCollect,
            )
        })
    }

    fn upload_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        origin: AccountIdFor<Self::T>,
        storage_deposit_limit: Option<BalanceOf<Self::T>>,
        determinism: Determinism,
    ) -> CodeUploadResult<<Self::T as frame_system::Config>::Hash, BalanceOf<Self::T>>
    {
        self.execute_with(|| {
            pallet_contracts::Pallet::<Self::T>::bare_upload_code(
                origin,
                contract_bytes,
                storage_deposit_limit,
                determinism,
            )
        })
    }

    fn call_contract(
        &mut self,
        address: AccountIdFor<Self::T>,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        origin: AccountIdFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: Option<BalanceOf<Self::T>>,
        determinism: Determinism,
    ) -> ContractExecResultFor<Self::T> {
        self.execute_with(|| {
            pallet_contracts::Pallet::<Self::T>::bare_call(
                origin,
                address,
                value,
                gas_limit,
                storage_deposit_limit,
                data,
                DebugInfo::UnsafeDebug,
                CollectEvents::UnsafeCollect,
                determinism,
            )
        })
    }
}

/// Converts bytes to a '\n'-split string, ignoring empty lines.
pub fn decode_debug_buffer(buffer: &[u8]) -> Vec<String> {
    let decoded = buffer.iter().map(|b| *b as char).collect::<String>();
    decoded
        .split('\n')
        .filter_map(|s| s.is_empty().not().then_some(s.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        prelude::*,
        DefaultSandbox,
        RuntimeEventOf,
        RuntimeOf,
    };
    use frame_support::sp_runtime::traits::Hash;
    use pallet_contracts::Origin;

    fn compile_module(contract_name: &str) -> Vec<u8> {
        let path = [
            std::env::var("CARGO_MANIFEST_DIR").as_deref().unwrap(),
            "/test-resources/",
            contract_name,
            ".wat",
        ]
        .concat();
        wat::parse_file(path).expect("Failed to parse wat file")
    }

    #[test]
    fn can_upload_code() {
        let mut sandbox = DefaultSandbox::default();
        let wasm_binary = compile_module("dummy");
        let hash = <<RuntimeOf<DefaultSandbox> as frame_system::Config>::Hashing>::hash(
            &wasm_binary,
        );

        let result = sandbox.upload_contract(
            wasm_binary,
            DefaultSandbox::default_actor(),
            None,
            Determinism::Enforced,
        );

        assert!(result.is_ok());
        assert_eq!(hash, result.unwrap().code_hash);
    }

    #[test]
    fn can_deploy_contract() {
        let mut sandbox = DefaultSandbox::default();
        let wasm_binary = compile_module("dummy");

        let events_before = sandbox.events();
        assert!(events_before.is_empty());

        let result = sandbox.deploy_contract(
            wasm_binary,
            0,
            vec![],
            vec![],
            DefaultSandbox::default_actor(),
            DefaultSandbox::default_gas_limit(),
            None,
        );
        assert!(result.result.is_ok());
        assert!(!result.result.unwrap().result.did_revert());

        let events = result.events.expect("Sandbox should collect events");
        let event_count = events.len();
        let instantiation_event = events[event_count - 2].clone();
        assert!(matches!(
            instantiation_event.event,
            RuntimeEventOf::<DefaultSandbox>::Contracts(pallet_contracts::Event::<
                RuntimeOf<DefaultSandbox>,
            >::Instantiated { .. })
        ));
        let deposit_event = events[event_count - 1].clone();
        assert!(matches!(
            deposit_event.event,
            RuntimeEventOf::<DefaultSandbox>::Contracts(
                pallet_contracts::Event::<RuntimeOf<DefaultSandbox>>::StorageDepositTransferredAndHeld { .. }
            )
        ));
    }

    #[test]
    fn can_call_contract() {
        let mut sandbox = DefaultSandbox::default();
        let actor = DefaultSandbox::default_actor();
        let wasm_binary = compile_module("dummy");

        let result = sandbox.deploy_contract(
            wasm_binary,
            0,
            vec![],
            vec![],
            actor.clone(),
            DefaultSandbox::default_gas_limit(),
            None,
        );

        let contract_address = result
            .result
            .expect("Contract should be deployed")
            .account_id;

        sandbox.reset_events();

        let result = sandbox.call_contract(
            contract_address.clone(),
            0,
            vec![],
            actor.clone(),
            DefaultSandbox::default_gas_limit(),
            None,
            Determinism::Enforced,
        );
        assert!(result.result.is_ok());
        assert!(!result.result.unwrap().did_revert());

        let events = result.events.expect("Sandbox should collect events");
        assert_eq!(events.len(), 2);

        assert_eq!(
            events[0].event,
            RuntimeEventOf::<DefaultSandbox>::Contracts(pallet_contracts::Event::<
                RuntimeOf<DefaultSandbox>,
            >::ContractEmitted {
                contract: contract_address.clone(),
                data: vec![0, 0, 0, 0],
            })
        );

        assert_eq!(
            events[1].event,
            RuntimeEventOf::<DefaultSandbox>::Contracts(pallet_contracts::Event::<
                RuntimeOf<DefaultSandbox>,
            >::Called {
                contract: contract_address,
                caller: Origin::Signed(actor),
            }),
        );
    }
}
