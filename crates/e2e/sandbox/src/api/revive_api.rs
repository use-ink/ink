use crate::{
    AccountIdFor,
    ContractExecResultFor,
    ContractResultInstantiate,
    Sandbox,
    H256,
};
use frame_support::{
    pallet_prelude::DispatchError,
    sp_runtime::traits::Bounded,
    traits::{
        fungible::Inspect,
        Time,
    },
    weights::Weight,
};
use frame_system::pallet_prelude::OriginFor;
use ink_primitives::DepositLimit;
use pallet_revive::{
    Code,
    CodeUploadResult,
    CollectEvents,
    DebugInfo,
};
use sp_core::{
    H160,
    U256,
};
use std::ops::Not;

type BalanceOf<R> =
    <<R as pallet_revive::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;

type MomentOf<T> = <<T as pallet_revive::Config>::Time as Time>::Moment;

/// Contract API used to interact with the contracts pallet.
pub trait ContractAPI {
    /// The runtime contract config.
    type T: pallet_revive::Config;

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
    fn map_account(&mut self, account: OriginFor<Self::T>) -> Result<(), DispatchError>;

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
        salt: Option<[u8; 32]>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<BalanceOf<Self::T>>,
    ) -> ContractResultInstantiate<Self::T>;

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
        code_hash: H256,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        salt: Option<[u8; 32]>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<BalanceOf<Self::T>>,
    ) -> ContractResultInstantiate<Self::T>;

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
        origin: OriginFor<Self::T>,
        storage_deposit_limit: BalanceOf<Self::T>,
    ) -> CodeUploadResult<BalanceOf<Self::T>>;

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
        address: H160,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<BalanceOf<Self::T>>,
    ) -> ContractExecResultFor<Self::T>;
}

impl<T> ContractAPI for T
where
    T: Sandbox,
    T::Runtime: pallet_revive::Config,

    BalanceOf<T::Runtime>: Into<U256> + TryFrom<U256> + Bounded,
    MomentOf<T::Runtime>: Into<U256>,
    <<T as Sandbox>::Runtime as frame_system::Config>::Hash:
        frame_support::traits::IsType<sp_core::H256>,
{
    type T = T::Runtime;

    fn map_account(
        &mut self,
        account_id: OriginFor<Self::T>,
    ) -> Result<(), DispatchError> {
        self.execute_with(|| pallet_revive::Pallet::<Self::T>::map_account(account_id))
    }

    fn deploy_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        salt: Option<[u8; 32]>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<BalanceOf<Self::T>>,
    ) -> ContractResultInstantiate<Self::T> {
        let storage_deposit_limit = storage_deposit_limit_fn(storage_deposit_limit);
        self.execute_with(|| {
            pallet_revive::Pallet::<Self::T>::bare_instantiate(
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
        code_hash: H256,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        salt: Option<[u8; 32]>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<BalanceOf<Self::T>>,
    ) -> ContractResultInstantiate<Self::T> {
        let storage_deposit_limit = storage_deposit_limit_fn(storage_deposit_limit);
        self.execute_with(|| {
            pallet_revive::Pallet::<Self::T>::bare_instantiate(
                origin,
                value,
                gas_limit,
                storage_deposit_limit,
                Code::Existing(code_hash),
                data,
                salt,
                DebugInfo::UnsafeDebug,
                CollectEvents::Skip,
            )
        })
    }

    fn upload_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        origin: OriginFor<Self::T>,
        storage_deposit_limit: BalanceOf<Self::T>,
    ) -> CodeUploadResult<BalanceOf<Self::T>> {
        self.execute_with(|| {
            pallet_revive::Pallet::<Self::T>::bare_upload_code(
                origin,
                contract_bytes,
                storage_deposit_limit,
            )
        })
    }

    fn call_contract(
        &mut self,
        address: H160,
        value: BalanceOf<Self::T>,
        data: Vec<u8>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<BalanceOf<Self::T>>,
    ) -> ContractExecResultFor<Self::T> {
        let storage_deposit_limit = storage_deposit_limit_fn(storage_deposit_limit);
        self.execute_with(|| {
            pallet_revive::Pallet::<Self::T>::bare_call(
                origin,
                address,
                value,
                gas_limit,
                storage_deposit_limit,
                data,
                DebugInfo::UnsafeDebug,
                CollectEvents::UnsafeCollect,
            )
        })
    }
}

/// todo
fn storage_deposit_limit_fn<Balance>(
    limit: DepositLimit<Balance>,
) -> pallet_revive::DepositLimit<Balance> {
    match limit {
        DepositLimit::Unchecked => pallet_revive::DepositLimit::Unchecked,
        DepositLimit::Balance(v) => pallet_revive::DepositLimit::Balance(v),
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
        api::prelude::*,
        DefaultSandbox,
        RuntimeEventOf,
        RuntimeOf,
    };

    const STORAGE_DEPOSIT_LIMIT: DepositLimit<u128> = DepositLimit::Unchecked;

    fn compile_module(contract_name: &str) -> Vec<u8> {
        let path = [
            std::env::var("CARGO_MANIFEST_DIR").as_deref().unwrap(),
            "/test-resources/",
            contract_name,
            ".polkavm",
        ]
        .concat();
        std::fs::read(std::path::Path::new(&path)).unwrap()
    }

    #[test]
    fn can_upload_code() {
        let mut sandbox = DefaultSandbox::default();
        let wasm_binary = compile_module("dummy");

        use sha3::{
            Digest,
            Keccak256,
        };
        let hash = Keccak256::digest(wasm_binary.as_slice());
        let hash = H256::from_slice(hash.as_slice());

        let origin =
            DefaultSandbox::convert_account_to_origin(DefaultSandbox::default_actor());
        let result = sandbox.upload_contract(wasm_binary, origin, 100000000000000);

        assert!(result.is_ok());
        assert_eq!(hash, result.unwrap().code_hash);
    }

    #[test]
    fn can_deploy_contract() {
        let mut sandbox = DefaultSandbox::default();
        let wasm_binary = compile_module("dummy");

        let events_before = sandbox.events();
        assert!(events_before.is_empty());

        let origin =
            DefaultSandbox::convert_account_to_origin(DefaultSandbox::default_actor());
        sandbox.map_account(origin.clone()).expect("cannot map");
        let result = sandbox.deploy_contract(
            wasm_binary,
            0,
            vec![],
            None,
            origin,
            DefaultSandbox::default_gas_limit(),
            DepositLimit::Balance(100000000000000),
        );
        assert!(result.result.is_ok());
        assert!(!result.result.unwrap().result.did_revert());

        let events = result.events.expect("Sandbox should collect events");
        let event_count = events.len();
        let instantiation_event = events[event_count - 2].clone();
        assert!(matches!(
            instantiation_event.event,
            RuntimeEventOf::<DefaultSandbox>::Revive(pallet_revive::Event::<
                RuntimeOf<DefaultSandbox>,
            >::Instantiated { .. })
        ));
        let deposit_event = events[event_count - 1].clone();
        assert!(matches!(
            deposit_event.event,
            RuntimeEventOf::<DefaultSandbox>::Revive(pallet_revive::Event::<
                RuntimeOf<DefaultSandbox>,
            >::StorageDepositTransferredAndHeld { .. })
        ));
    }

    #[test]
    fn can_call_contract() {
        let mut sandbox = DefaultSandbox::default();
        let _actor = DefaultSandbox::default_actor();
        let wasm_binary = compile_module("dummy");

        let origin =
            DefaultSandbox::convert_account_to_origin(DefaultSandbox::default_actor());
        sandbox.map_account(origin.clone()).expect("unable to map");
        let result = sandbox.deploy_contract(
            wasm_binary,
            0,
            vec![],
            None,
            origin.clone(),
            DefaultSandbox::default_gas_limit(),
            STORAGE_DEPOSIT_LIMIT,
        );
        assert!(!result.result.clone().unwrap().result.did_revert());

        let contract_address = result.result.expect("Contract should be deployed").addr;

        sandbox.reset_events();

        let result = sandbox.call_contract(
            contract_address,
            0,
            vec![],
            origin.clone(),
            DefaultSandbox::default_gas_limit(),
            STORAGE_DEPOSIT_LIMIT,
        );
        assert!(result.result.is_ok());
        assert!(!result.result.unwrap().did_revert());

        let events = result.events.expect("Sandbox should collect events");
        assert_eq!(events.len(), 2);

        assert_eq!(
            events[0].event,
            RuntimeEventOf::<DefaultSandbox>::Revive(pallet_revive::Event::<
                RuntimeOf<DefaultSandbox>,
            >::ContractEmitted {
                contract: contract_address,
                topics: vec![H256::from([42u8; 32])],
                data: vec![1, 2, 3, 4],
            })
        );

        // TODO Wait for `pallet_revive::exec::Origin` re-export.
        // let account_id = DefaultSandbox::default_actor();
        // let caller = origin.clone();
        // let caller = pallet_revive::exec::Origin::from_runtime_origin(caller).unwrap();
        // let origin =
        // DefaultSandbox::convert_account_to_origin(DefaultSandbox::default_actor());
        // let foo = pallet_revive::Origin::<RuntimeOf<DefaultSandbox>>::from(origin);
        // assert_eq!(
        // events[1].event,
        // RuntimeEventOf::<DefaultSandbox>::Revive(pallet_revive::Event::<
        // RuntimeOf<DefaultSandbox>,
        // >::Called {
        // contract: contract_address,
        // caller: frame_system::EnsureSigned::try_origin(actor).unwrap(),
        // caller,
        // }),
        // );
    }
}
