// Copyright (C) Use Ink (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    backend::BuilderClient,
    builders::{
        constructor_exec_input,
        CreateBuilderPartial,
    },
    client_utils::{
        salt,
        ContractsRegistry,
    },
    contract_results::BareInstantiationResult,
    error::SandboxErr,
    log_error,
    CallBuilderFinal,
    CallDryRunResult,
    ChainBackend,
    ContractsBackend,
    E2EBackend,
    InstantiateDryRunResult,
    UploadResult,
};

use frame_support::traits::fungible::Inspect;
use ink_sandbox::{
    api::prelude::*,
    pallet_balances,
    pallet_contracts,
    AccountIdFor,
    RuntimeCall,
    Sandbox,
    Weight,
};
use pallet_contracts::ContractResult;

use ink_env::Environment;
use jsonrpsee::core::async_trait;
use pallet_contracts::{
    CodeUploadReturnValue,
    ContractInstantiateResult,
    InstantiateReturnValue,
};
use scale::{
    Decode,
    Encode,
};
use sp_core::{
    sr25519::Pair,
    Pair as _,
};
use std::{
    marker::PhantomData,
    path::PathBuf,
};
use subxt::{
    dynamic::Value,
    tx::Payload,
};
use subxt_signer::sr25519::Keypair;

type BalanceOf<R> = <R as pallet_balances::Config>::Balance;
type ContractsBalanceOf<R> =
    <<R as pallet_contracts::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;

pub struct Client<AccountId, Hash, S: Sandbox> {
    sandbox: S,
    contracts: ContractsRegistry,
    _phantom: PhantomData<(AccountId, Hash)>,
}

// While it is not necessary true that `Client` is `Send`, it will not be used in a way
// that would violate this bound. In particular, all `Client` instances will be operating
// synchronously.
unsafe impl<AccountId, Hash, S: Sandbox> Send for Client<AccountId, Hash, S> {}
impl<AccountId, Hash, S: Sandbox> Client<AccountId, Hash, S>
where
    S: Default,
    S::Runtime: pallet_balances::Config + pallet_contracts::Config,
    AccountIdFor<S::Runtime>: From<[u8; 32]>,
    BalanceOf<S::Runtime>: From<u128>,
{
    pub fn new<P: Into<PathBuf>>(contracts: impl IntoIterator<Item = P>) -> Self {
        let mut sandbox = S::default();
        Self::fund_accounts(&mut sandbox);

        Self {
            sandbox,
            contracts: ContractsRegistry::new(contracts),
            _phantom: Default::default(),
        }
    }

    fn fund_accounts(sandbox: &mut S) {
        const TOKENS: u128 = 1_000_000_000_000_000;

        let accounts = [
            crate::alice(),
            crate::bob(),
            crate::charlie(),
            crate::dave(),
            crate::eve(),
            crate::ferdie(),
            crate::one(),
            crate::two(),
        ]
        .map(|kp| kp.public_key().0)
        .map(From::from);
        for account in accounts.iter() {
            sandbox
                .mint_into(account, TOKENS.into())
                .unwrap_or_else(|_| panic!("Failed to mint {} tokens", TOKENS));
        }
    }
}

#[async_trait]
impl<AccountId: AsRef<[u8; 32]> + Send, Hash, S: Sandbox> ChainBackend
    for Client<AccountId, Hash, S>
where
    S::Runtime: pallet_balances::Config,
    AccountIdFor<S::Runtime>: From<[u8; 32]>,
{
    type AccountId = AccountId;
    type Balance = BalanceOf<S::Runtime>;
    type Error = SandboxErr;
    type EventLog = ();

    async fn create_and_fund_account(
        &mut self,
        _origin: &Keypair,
        amount: Self::Balance,
    ) -> Keypair {
        let (pair, seed) = Pair::generate();

        self.sandbox
            .mint_into(&pair.public().0.into(), amount)
            .expect("Failed to mint tokens");

        Keypair::from_secret_key(seed).expect("Failed to create keypair")
    }

    async fn free_balance(
        &mut self,
        account: Self::AccountId,
    ) -> Result<Self::Balance, Self::Error> {
        let account = AccountIdFor::<S::Runtime>::from(*account.as_ref());
        Ok(self.sandbox.free_balance(&account))
    }

    async fn runtime_call<'a>(
        &mut self,
        origin: &Keypair,
        pallet_name: &'a str,
        call_name: &'a str,
        call_data: Vec<Value>,
    ) -> Result<Self::EventLog, Self::Error> {
        // Since in general, `ChainBackend::runtime_call` must be dynamic, we have to
        // perform some translation here in order to invoke strongly-typed
        // [`ink_sandbox::Sandbox`] API.

        // Get metadata of the Sandbox runtime, so that we can encode the call object.
        // Panic on error - metadata of the static im-memory runtime should always be
        // available.
        let raw_metadata: Vec<u8> = S::get_metadata().into();
        let metadata = subxt_metadata::Metadata::decode(&mut raw_metadata.as_slice())
            .expect("Failed to decode metadata");

        // Encode the call object.
        let call = subxt::dynamic::tx(pallet_name, call_name, call_data);
        let encoded_call = call.encode_call_data(&metadata.into()).map_err(|err| {
            SandboxErr::new(format!("runtime_call: Error encoding call: {err:?}"))
        })?;

        // Decode the call object.
        // Panic on error - we just encoded a validated call object, so it should be
        // decodable.
        let decoded_call =
            RuntimeCall::<S::Runtime>::decode(&mut encoded_call.as_slice())
                .expect("Failed to decode runtime call");

        // Execute the call.
        self.sandbox
            .runtime_call(
                decoded_call,
                S::convert_account_to_origin(keypair_to_account(origin)),
            )
            .map_err(|err| {
                SandboxErr::new(format!("runtime_call: execution error {:?}", err.error))
            })?;

        Ok(())
    }
}

#[async_trait]
impl<
        AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
        Hash: Copy + Send + From<[u8; 32]>,
        S: Sandbox,
        E: Environment<
                AccountId = AccountId,
                Balance = ContractsBalanceOf<S::Runtime>,
                Hash = Hash,
            > + 'static,
    > BuilderClient<E> for Client<AccountId, Hash, S>
where
    S::Runtime: pallet_balances::Config + pallet_contracts::Config,
    AccountIdFor<S::Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
    ContractsBalanceOf<S::Runtime>: Send + Sync,
{
    async fn bare_instantiate<Contract: Clone, Args: Send + Sync + Encode + Clone, R>(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        let data = constructor_exec_input(constructor.clone());

        let result = self.sandbox.deploy_contract(
            code,
            value,
            data,
            salt(),
            keypair_to_account(caller),
            gas_limit,
            storage_deposit_limit,
        );

        let account_id_raw = match &result.result {
            Err(err) => {
                log_error(&format!("Instantiation failed: {err:?}"));
                return Err(SandboxErr::new(format!("bare_instantiate: {err:?}")));
            }
            Ok(res) => *res.account_id.as_ref(),
        };
        let account_id = AccountId::from(account_id_raw);

        Ok(BareInstantiationResult {
            account_id: account_id.clone(),
            events: (), // todo: https://github.com/Cardinal-Cryptography/drink/issues/32
        })
    }

    async fn bare_instantiate_dry_run<Contract: Clone, Args: Send + Encode + Clone, R>(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiateDryRunResult<E>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        let data = constructor_exec_input(constructor.clone());
        let result = self.sandbox.dry_run(|sandbox| {
            sandbox.deploy_contract(
                code,
                value,
                data,
                salt(),
                keypair_to_account(caller),
                S::default_gas_limit(),
                storage_deposit_limit,
            )
        });

        let account_id_raw = match &result.result {
            Err(err) => {
                panic!("Instantiate dry-run failed: {err:?}!")
            }
            Ok(res) => *res.account_id.as_ref(),
        };
        let account_id = AccountId::from(account_id_raw);

        let result = ContractInstantiateResult {
            gas_consumed: result.gas_consumed,
            gas_required: result.gas_required,
            storage_deposit: result.storage_deposit,
            debug_message: result.debug_message,
            result: result.result.map(|r| {
                InstantiateReturnValue {
                    result: r.result,
                    account_id,
                }
            }),
            events: None,
        };
        Ok(result.into())
    }

    async fn bare_upload(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<UploadResult<E, Self::EventLog>, Self::Error> {
        let code = self.contracts.load_code(contract_name);

        let result = match self.sandbox.upload_contract(
            code,
            keypair_to_account(caller),
            storage_deposit_limit,
            pallet_contracts::Determinism::Enforced,
        ) {
            Ok(result) => result,
            Err(err) => {
                log_error(&format!("Upload failed: {err:?}"));
                return Err(SandboxErr::new(format!("bare_upload: {err:?}")))
            }
        };

        let code_hash_raw: [u8; 32] = result
            .code_hash
            .as_ref()
            .try_into()
            .expect("Invalid code hash");
        let code_hash = Hash::from(code_hash_raw);
        Ok(UploadResult {
            code_hash,
            dry_run: Ok(CodeUploadReturnValue {
                code_hash,
                deposit: result.deposit,
            }),
            events: (),
        })
    }

    async fn bare_remove_code(
        &mut self,
        _caller: &Keypair,
        _code_hash: E::Hash,
    ) -> Result<Self::EventLog, Self::Error> {
        unimplemented!("sandbox does not yet support remove_code")
    }

    async fn bare_call<Args: Sync + Encode + Clone, RetType: Send + Decode>(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<Self::EventLog, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        let account_id = message.clone().params().callee().clone();
        let exec_input = Encode::encode(message.clone().params().exec_input());
        let account_id = (*account_id.as_ref()).into();

        self.sandbox
            .call_contract(
                account_id,
                value,
                exec_input,
                keypair_to_account(caller),
                gas_limit,
                storage_deposit_limit,
                pallet_contracts::Determinism::Enforced,
            )
            .result
            .map_err(|err| SandboxErr::new(format!("bare_call: {err:?}")))?;

        Ok(())
    }

    async fn bare_call_dry_run<Args: Sync + Encode + Clone, RetType: Send + Decode>(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<CallDryRunResult<E, RetType>, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        let account_id = message.clone().params().callee().clone();
        let exec_input = Encode::encode(message.clone().params().exec_input());
        let account_id = (*account_id.as_ref()).into();

        let result = self.sandbox.dry_run(|sandbox| {
            sandbox.call_contract(
                account_id,
                value,
                exec_input,
                keypair_to_account(caller),
                S::default_gas_limit(),
                storage_deposit_limit,
                pallet_contracts::Determinism::Enforced,
            )
        });
        Ok(CallDryRunResult {
            exec_result: ContractResult {
                gas_consumed: result.gas_consumed,
                gas_required: result.gas_required,
                storage_deposit: result.storage_deposit,
                debug_message: result.debug_message,
                result: result.result,
                events: None,
            },
            _marker: Default::default(),
        })
    }
}

impl<
        AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
        Hash: Copy + Send + From<[u8; 32]>,
        Config: Sandbox,
        E: Environment<
                AccountId = AccountId,
                Balance = ContractsBalanceOf<Config::Runtime>,
                Hash = Hash,
            > + 'static,
    > E2EBackend<E> for Client<AccountId, Hash, Config>
where
    Config::Runtime: pallet_balances::Config + pallet_contracts::Config,
    AccountIdFor<Config::Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
    ContractsBalanceOf<Config::Runtime>: Send + Sync,
{
}

fn keypair_to_account<AccountId: From<[u8; 32]>>(keypair: &Keypair) -> AccountId {
    AccountId::from(keypair.public_key().0)
}

#[async_trait]
impl<
        AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
        Hash: Copy + From<[u8; 32]>,
        S: Sandbox,
        E: Environment<
                AccountId = AccountId,
                Balance = ContractsBalanceOf<S::Runtime>,
                Hash = Hash,
            > + 'static,
    > ContractsBackend<E> for Client<AccountId, Hash, S>
where
    S::Runtime: pallet_balances::Config + pallet_contracts::Config,
    AccountIdFor<S::Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
{
    type Error = SandboxErr;
    type EventLog = ();
}

/// Exposes preset sandbox configurations to be used in tests.
pub mod preset {
    pub mod mock_network {
        use ink_sandbox::{
            frame_system,
            AccountIdFor,
            BlockBuilder,
            Extension,
            RuntimeMetadataPrefixed,
            Sandbox,
            Snapshot,
        };
        pub use pallet_contracts_mock_network::*;
        use sp_runtime::traits::Dispatchable;

        /// A [`ink_sandbox::Sandbox`] that can be used to test contracts
        /// with a mock network of relay chain and parachains.
        ///
        /// ```no_compile
        /// #[ink_e2e::test(backend(runtime_only(sandbox = MockNetworkSandbox)))]
        /// async fn my_test<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        ///   // ...
        /// }
        /// ```
        #[derive(Default)]
        pub struct MockNetworkSandbox {
            dry_run: bool,
        }

        impl Sandbox for MockNetworkSandbox {
            type Runtime = parachain::Runtime;

            fn execute_with<T>(&mut self, execute: impl FnOnce() -> T) -> T {
                if self.dry_run {
                    ParaA::execute_with(execute)
                } else {
                    ParaA::execute_without_dispatch(execute)
                }
            }

            fn dry_run<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T {
                EXT_PARAA.with(|v| {
                    let backend_backup = v.borrow_mut().as_backend();
                    self.dry_run = true;
                    let result = action(self);
                    self.dry_run = false;

                    let mut v = v.borrow_mut();
                    v.commit_all().expect("Failed to commit changes");
                    v.backend = backend_backup;
                    result
                })
            }

            fn register_extension<E: ::core::any::Any + Extension>(&mut self, ext: E) {
                EXT_PARAA.with(|v| v.borrow_mut().register_extension(ext));
            }

            fn initialize_block(
                height: frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
                parent_hash: <Self::Runtime as frame_system::Config>::Hash,
            ) {
                BlockBuilder::<Self::Runtime>::initialize_block(height, parent_hash)
            }

            fn finalize_block(
                height: frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
            ) -> <Self::Runtime as frame_system::Config>::Hash {
                BlockBuilder::<Self::Runtime>::finalize_block(height)
            }

            fn default_actor() -> AccountIdFor<Self::Runtime> {
                ALICE
            }

            fn get_metadata() -> RuntimeMetadataPrefixed {
                parachain::Runtime::metadata()
            }

            fn convert_account_to_origin(
                account: AccountIdFor<Self::Runtime>,
            ) -> <<Self::Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin
            {
                Some(account).into()
            }

            fn take_snapshot(&mut self) -> Snapshot {
                EXT_PARAA.with(|v| {
                    let mut v = v.borrow_mut();
                    let mut backend = v.as_backend().clone();
                    let raw_key_values = backend
                        .backend_storage_mut()
                        .drain()
                        .into_iter()
                        .filter(|(_, (_, r))| *r > 0)
                        .collect::<Vec<(Vec<u8>, (Vec<u8>, i32))>>();
                    let root = backend.root().to_owned();

                    Snapshot {
                        storage: raw_key_values,
                        storage_root: root,
                    }
                })
            }

            fn restore_snapshot(&mut self, snapshot: ink_sandbox::Snapshot) {
                EXT_PARAA.with(|v| {
                    let mut v = v.borrow_mut();

                    *v = ink_sandbox::TestExternalities::from_raw_snapshot(
                        snapshot.storage,
                        snapshot.storage_root,
                        Default::default(),
                    );
                })
            }
        }
    }
}
