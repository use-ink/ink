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
    AccountIdFor,
    RuntimeCall,
    Sandbox,
    api::prelude::*,
    error::SandboxErr,
    frame_system,
    frame_system::pallet_prelude::OriginFor,
    pallet_balances,
    pallet_revive,
    pallet_revive::{
        AddressMapper,
        MomentOf,
        evm::{
            CallTracerConfig,
            Trace,
            TracerType,
        },
    },
    to_revive_storage_deposit,
    to_revive_trace,
};
use frame_support::{
    dispatch::RawOrigin,
    pallet_prelude::DispatchError,
    traits::{
        IsType,
        fungible::Inspect,
    },
};
use ink_e2e::{
    BareInstantiationResult,
    BuilderClient,
    CallBuilderFinal,
    CallDryRunResult,
    ChainBackend,
    ContractExecResultFor,
    ContractResult,
    ContractsBackend,
    ContractsRegistry,
    CreateBuilderPartial,
    E2EBackend,
    InstantiateDryRunResult,
    UploadResult,
    constructor_exec_input,
    keypair_to_account,
    log_error,
    salt,
    subxt::{
        self,
        dynamic::Value,
        tx::Payload,
    },
    subxt_signer::sr25519::{
        Keypair,
        dev,
    },
};
use ink_env::{
    Environment,
    call::utils::DecodeMessageResult,
};
use ink_primitives::{
    H160,
    H256,
    U256,
    Weight,
    abi::AbiEncodeWith,
};
use ink_revive_types::{
    CodeUploadReturnValue,
    ExecReturnValue,
    InstantiateReturnValue,
    evm::CallTrace,
};
use jsonrpsee::core::async_trait;
use scale::Decode;
use sp_core::{
    Pair as _,
    sr25519::Pair,
};
use sp_runtime::traits::Bounded;
use std::{
    marker::PhantomData,
    path::PathBuf,
};

type BalanceOf<R> = <R as pallet_balances::Config>::Balance;
type ContractsBalanceOf<R> =
    <<R as pallet_revive::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;

pub struct Client<AccountId, S: Sandbox> {
    sandbox: S,
    contracts: ContractsRegistry,
    _phantom: PhantomData<AccountId>,
}

// While it is not necessary true that `Client` is `Send`, it will not be used in a way
// that would violate this bound. In particular, all `Client` instances will be operating
// synchronously.
unsafe impl<AccountId, S: Sandbox> Send for Client<AccountId, S> {}
impl<AccountId, S: Sandbox> Client<AccountId, S>
where
    S: Default,
    S::Runtime: pallet_balances::Config + pallet_revive::Config,
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
            dev::alice(),
            dev::bob(),
            dev::charlie(),
            dev::dave(),
            dev::eve(),
            dev::ferdie(),
            dev::one(),
            dev::two(),
        ]
        .map(|kp| kp.public_key().0)
        .map(From::from);
        for account in accounts.iter() {
            sandbox
                .mint_into(account, TOKENS.into())
                .unwrap_or_else(|_| panic!("Failed to mint {TOKENS} tokens"));
        }

        let acc = pallet_revive::Pallet::<S::Runtime>::account_id();
        let ed = pallet_balances::Pallet::<S::Runtime>::minimum_balance();
        sandbox.mint_into(&acc, ed).unwrap_or_else(|_| {
            panic!("Failed to mint existential deposit into `pallet-revive` account")
        });
    }
}

#[async_trait]
impl<AccountId: AsRef<[u8; 32]> + Send, S: Sandbox> ChainBackend for Client<AccountId, S>
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

    async fn transfer_allow_death(
        &mut self,
        origin: &Keypair,
        dest: Self::AccountId,
        value: Self::Balance,
    ) -> Result<(), Self::Error> {
        let caller = keypair_to_account(origin);
        let origin = RawOrigin::Signed(caller);
        let origin = OriginFor::<S::Runtime>::from(origin);

        let dest = dest.as_ref();
        let dest = Decode::decode(&mut dest.as_slice()).unwrap();

        self.sandbox
            .transfer_allow_death(&origin, &dest, value)
            .map_err(|err| {
                SandboxErr::new(format!("transfer_allow_death failed: {err:?}"))
            })
    }
}

#[async_trait]
impl<
    AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
    S: Sandbox,
    E: Environment<AccountId = AccountId, Balance = ContractsBalanceOf<S::Runtime>>
        + 'static,
> BuilderClient<E> for Client<AccountId, S>
where
    S::Runtime: pallet_balances::Config + pallet_revive::Config,
    AccountIdFor<S::Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
    ContractsBalanceOf<S::Runtime>: Send + Sync,
    ContractsBalanceOf<S::Runtime>: Into<U256> + TryFrom<U256> + Bounded,
    MomentOf<S::Runtime>: Into<U256>,
    <<S as Sandbox>::Runtime as frame_system::Config>::Nonce: Into<u32>,
    // todo
    <<S as Sandbox>::Runtime as frame_system::Config>::Hash:
        frame_support::traits::IsType<sp_core::H256>,
{
    fn load_code(&self, contract_name: &str) -> Vec<u8> {
        self.contracts.load_code(contract_name)
    }

    async fn exec_instantiate(
        &mut self,
        signer: &Keypair,
        contract_name: &str,
        data: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: E::Balance,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        self.raw_instantiate(code, signer, data, value, gas_limit, storage_deposit_limit)
            .await
    }

    async fn bare_instantiate<
        Contract: Clone,
        Args: Send + Sync + AbiEncodeWith<Abi> + Clone,
        R,
        Abi: Send + Sync + Clone,
    >(
        &mut self,
        code: Vec<u8>,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R, Abi>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: E::Balance,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error> {
        let data = constructor_exec_input(constructor.clone());
        self.raw_instantiate(code, caller, data, value, gas_limit, storage_deposit_limit)
            .await
    }

    async fn raw_instantiate(
        &mut self,
        code: Vec<u8>,
        caller: &Keypair,
        data: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: E::Balance,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error> {
        let _ =
            <Client<AccountId, S> as BuilderClient<E>>::map_account(self, caller).await;

        // get the trace
        let _ = self.sandbox.build_block();

        let tracer_type = TracerType::CallTracer(Some(CallTracerConfig::default()));
        let mut tracer = self.sandbox.evm_tracer(tracer_type);

        let mut code_hash: Option<H256> = None;
        let result = pallet_revive::tracing::trace(tracer.as_tracing(), || {
            code_hash = Some(H256(ink_e2e::code_hash(&code[..])));
            self.sandbox.deploy_contract(
                code,
                value,
                data,
                salt(),
                caller_to_origin::<S>(caller),
                // TODO: mismatch in dependencies
                pallet_revive::Weight::from_parts(
                    gas_limit.ref_time(),
                    gas_limit.proof_size(),
                ),
                storage_deposit_limit,
            )
        });

        let addr_raw = match &result.result {
            Err(err) => {
                log_error(&format!("instantiation failed: {err:?}"));
                return Err(SandboxErr::new(format!("bare_instantiate: {err:?}")));
            }
            Ok(res) => res.addr,
        };

        let trace = match tracer.collect_trace() {
            Some(Trace::Call(call_trace)) => Some(to_revive_trace(call_trace)),
            _ => None,
        };

        let account_id =
            <S::Runtime as pallet_revive::Config>::AddressMapper::to_fallback_account_id(
                &addr_raw,
            )
            .as_ref()
            .to_owned();

        Ok(BareInstantiationResult {
            addr: addr_raw,
            account_id: account_id.into(),
            events: (), // todo: https://github.com/Cardinal-Cryptography/drink/issues/32
            trace,
            code_hash: code_hash.expect("code_hash must have been calculated"),
        })
    }

    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn bare_instantiate_dry_run<
        Contract: Clone,
        Args: Send + Sync + AbiEncodeWith<Abi> + Clone,
        R,
        Abi: Send + Sync + Clone,
    >(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R, Abi>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiateDryRunResult<E, Abi>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        let exec_input = constructor.clone().params().exec_input().encode();
        self.raw_instantiate_dry_run(
            code,
            caller,
            exec_input,
            value,
            storage_deposit_limit,
        )
        .await
    }

    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn raw_instantiate_dry_run<Abi: Sync + Clone>(
        &mut self,
        code: Vec<u8>,
        caller: &Keypair,
        data: Vec<u8>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiateDryRunResult<E, Abi>, Self::Error> {
        // There's a side effect here!
        let _ =
            <Client<AccountId, S> as BuilderClient<E>>::map_account(self, caller).await;

        let dry_run_result = self.sandbox.dry_run(|sandbox| {
            sandbox.deploy_contract(
                code,
                value,
                data,
                salt(),
                caller_to_origin::<S>(caller),
                S::default_gas_limit(),
                storage_deposit_limit.unwrap_or(E::Balance::max_value()),
            )
        });

        if let Err(err) = dry_run_result.result {
            panic!("Instantiate dry-run failed: {err:?}!")
        };

        let result = ContractResult::<InstantiateReturnValue, E::Balance> {
            // TODO: mismatch in dependencies
            gas_consumed: Weight::from_parts(
                dry_run_result.gas_consumed.ref_time(),
                dry_run_result.gas_consumed.proof_size(),
            ),
            gas_required: Weight::from_parts(
                dry_run_result.gas_required.ref_time(),
                dry_run_result.gas_required.proof_size(),
            ),
            storage_deposit: to_revive_storage_deposit(dry_run_result.storage_deposit),
            result: dry_run_result
                .result
                .map_err(|_e| sp_runtime::DispatchError::Other("SandboxError")) // TODO: mismatch in dependencies
                .map(|res| {
                    InstantiateReturnValue {
                        result: ExecReturnValue {
                            // TODO: mismatch in dependencies
                            flags: ink_env::ReturnFlags::from_bits_truncate(res.result.flags.bits()),
                            data: res.result.data,
                        },
                        addr: res.addr,
                    }
                }),
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
            caller_to_origin::<S>(caller),
            storage_deposit_limit.unwrap_or_else(|| E::Balance::max_value()),
        ) {
            Ok(result) => result,
            Err(err) => {
                log_error(&format!("upload failed: {err:?}"));
                return Err(SandboxErr::new(format!("bare_upload: {err:?}")))
            }
        };

        Ok(UploadResult {
            code_hash: result.code_hash,
            dry_run: Ok(CodeUploadReturnValue {
                code_hash: result.code_hash,
                deposit: result.deposit,
            }),
            events: (),
        })
    }

    async fn bare_remove_code(
        &mut self,
        _caller: &Keypair,
        _code_hash: H256,
    ) -> Result<Self::EventLog, Self::Error> {
        unimplemented!("sandbox does not yet support remove_code")
    }

    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn bare_call<
        Args: Sync + AbiEncodeWith<Abi> + Clone,
        RetType: Send + DecodeMessageResult<Abi>,
        Abi: Sync + Clone,
    >(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType, Abi>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: E::Balance,
    ) -> Result<(Self::EventLog, Option<CallTrace>), Self::Error>
    where
        CallBuilderFinal<E, Args, RetType, Abi>: Clone,
    {
        // There's a side effect here!
        let _ =
            <Client<AccountId, S> as BuilderClient<E>>::map_account(self, caller).await;

        let addr = *message.clone().params().callee();
        let exec_input = message.clone().params().exec_input().encode();
        <Client<AccountId, S> as BuilderClient<E>>::raw_call::<'_, '_, '_>(
            self,
            addr,
            exec_input,
            value,
            gas_limit,
            storage_deposit_limit,
            caller,
        )
        .await
    }

    async fn raw_call(
        &mut self,
        dest: H160,
        input_data: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: E::Balance,
        signer: &Keypair,
    ) -> Result<(Self::EventLog, Option<CallTrace>), Self::Error> {
        // todo
        let tracer_type = TracerType::CallTracer(Some(CallTracerConfig::default()));
        let mut tracer = self.sandbox.evm_tracer(tracer_type);
        let _result = pallet_revive::tracing::trace(tracer.as_tracing(), || {
            self.sandbox
                .call_contract(
                    dest,
                    value,
                    input_data,
                    caller_to_origin::<S>(signer),
                    // TODO: mismatch in dependencies
                    pallet_revive::Weight::from_parts(
                        gas_limit.ref_time(),
                        gas_limit.proof_size(),
                    ),
                    storage_deposit_limit,
                )
                .result
                .map_err(|err| SandboxErr::new(format!("bare_call: {err:?}")))
        })?;
        let trace = match tracer.collect_trace() {
            Some(Trace::Call(call_trace)) => Some(to_revive_trace(call_trace)),
            _ => None,
        };

        Ok(((), trace))
    }

    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn bare_call_dry_run<
        Args: Sync + AbiEncodeWith<Abi> + Clone,
        RetType: Send + DecodeMessageResult<Abi>,
        Abi: Sync + Clone,
    >(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType, Abi>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<CallDryRunResult<E, RetType, Abi>, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType, Abi>: Clone,
    {
        let addr = *message.clone().params().callee();
        let exec_input = message.clone().params().exec_input().encode();
        self.raw_call_dry_run(addr, exec_input, value, storage_deposit_limit, caller)
            .await
    }

    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn raw_call_dry_run<
        RetType: Send + DecodeMessageResult<Abi>,
        Abi: Sync + Clone,
    >(
        &mut self,
        dest: H160,
        input_data: Vec<u8>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
        caller: &Keypair,
    ) -> Result<CallDryRunResult<E, RetType, Abi>, Self::Error> {
        // There's a side effect here!
        let _ =
            <Client<AccountId, S> as BuilderClient<E>>::map_account(self, caller).await;

        let result = self.sandbox.dry_run(|sandbox| {
            sandbox.call_contract(
                dest,
                value,
                input_data,
                caller_to_origin::<S>(caller),
                S::default_gas_limit(),
                storage_deposit_limit.unwrap_or(E::Balance::max_value()),
            )
        });
        if result.result.is_err() {
            let res = result.result.clone().unwrap_err();
            if let DispatchError::Module(m) = res
                && let Some(s) = m.message
                && s.contains("AccountUnmapped")
            {
                panic!("something is wrong, we mapped the account before")
            }
        }
        // todo error when `AccountUnmapped`
        Ok(CallDryRunResult {
            exec_result: ContractExecResultFor::<E> {
                // TODO: mismatch in dependencies
                gas_consumed: Weight::from_parts(
                    result.gas_consumed.ref_time(),
                    result.gas_consumed.proof_size(),
                ),
                gas_required: Weight::from_parts(
                    result.gas_required.ref_time(),
                    result.gas_required.proof_size(),
                ),
                storage_deposit: to_revive_storage_deposit(result.storage_deposit),
                result: result
                    .result
                    .map_err(|_e| sp_runtime::DispatchError::Other("SandboxError")) // TODO: mismatch in dependencies
                    .map(|res| {
                        ExecReturnValue {
                            // TODO: mismatch in dependencies
                            flags: ink_env::ReturnFlags::from_bits_truncate(res.flags.bits()),
                            data: res.data,
                        }
                    }),
            },
            trace: None, // todo
            _marker: Default::default(),
        })
    }

    async fn map_account(
        &mut self,
        caller: &Keypair,
    ) -> Result<Option<Self::EventLog>, Self::Error> {
        let caller = keypair_to_account(caller);
        let origin = RawOrigin::Signed(caller);
        let origin = OriginFor::<S::Runtime>::from(origin);

        self.sandbox
            .map_account(origin)
            .map_err(|err| {
                SandboxErr::new(format!("map_account: execution error {err:?}"))
            })
            .map(|_| None)
    }

    async fn to_account_id(&mut self, addr: &H160) -> Result<E::AccountId, Self::Error> {
        use pallet_revive::AddressMapper;
        let account_id =
            <S::Runtime as pallet_revive::Config>::AddressMapper::to_account_id(addr);
        Ok(E::AccountId::from(*account_id.as_ref()))
    }
}

impl<
    AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
    Config: Sandbox,
    E: Environment<AccountId = AccountId, Balance = ContractsBalanceOf<Config::Runtime>>
        + 'static,
> E2EBackend<E> for Client<AccountId, Config>
where
    Config::Runtime: pallet_balances::Config + pallet_revive::Config,
    AccountIdFor<Config::Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
    ContractsBalanceOf<Config::Runtime>: Send + Sync,
    ContractsBalanceOf<Config::Runtime>: Into<U256> + TryFrom<U256> + Bounded,
    MomentOf<Config::Runtime>: Into<U256>,
    <Config::Runtime as frame_system::Config>::Nonce: Into<u32>,
    // todo
    <Config::Runtime as frame_system::Config>::Hash: IsType<sp_core::H256>,
{
}

#[async_trait]
impl<
    AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
    S: Sandbox,
    E: Environment<AccountId = AccountId, Balance = ContractsBalanceOf<S::Runtime>>
        + 'static,
> ContractsBackend<E> for Client<AccountId, S>
where
    S::Runtime: pallet_balances::Config + pallet_revive::Config,
    AccountIdFor<S::Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
{
    type Error = SandboxErr;
    type EventLog = ();
}

/// Exposes preset sandbox configurations to be used in tests.
pub mod preset {
    /*
    // todo
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
        pub use pallet_revive_mock_network::*;
        use sp_runtime::traits::Dispatchable;

        /// A [`ink_sandbox::Sandbox`] that can be used to test contracts
        /// with a mock network of relay chain and parachains.
        ///
        /// ```no_compile
        /// #[ink_e2e::test(backend(runtime_only(sandbox = MockNetworkSandbox, client = ink_sandbox::SandboxClient)))]
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
     */
}

/// Transforms a `Keypair` into an origin.
pub fn caller_to_origin<S>(caller: &Keypair) -> OriginFor<S::Runtime>
where
    S: Sandbox,
    S::Runtime: pallet_balances::Config + pallet_revive::Config,
    AccountIdFor<S::Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
{
    let caller = keypair_to_account(caller);
    let origin = RawOrigin::Signed(caller);
    OriginFor::<S::Runtime>::from(origin)
}
