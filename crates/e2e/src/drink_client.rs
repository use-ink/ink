// Copyright (C) Parity Technologies (UK) Ltd.
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
    error::DrinkErr,
    log_error,
    CallBuilderFinal,
    CallDryRunResult,
    ChainBackend,
    ContractsBackend,
    E2EBackend,
    InstantiateDryRunResult,
    UploadResult,
};
use drink::{
    frame_support::traits::fungible::Inspect,
    pallet_balances,
    pallet_contracts,
    runtime::{
        AccountIdFor,
        Runtime as RuntimeT,
    },
    BalanceOf,
    RuntimeCall,
    Sandbox,
    Weight,
    DEFAULT_GAS_LIMIT,
};
use pallet_contracts_primitives::ContractResult;

use ink_env::Environment;
use jsonrpsee::core::async_trait;
use pallet_contracts_primitives::{
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
    tx::TxPayload,
};
use subxt_signer::sr25519::Keypair;

type ContractsBalanceOf<R> =
    <<R as pallet_contracts::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;
pub struct Client<AccountId, Hash, Runtime: RuntimeT> {
    sandbox: Sandbox<Runtime>,
    contracts: ContractsRegistry,
    _phantom: PhantomData<(AccountId, Hash)>,
}

// While it is not necessary true that `Client` is `Send`, it will not be used in a way
// that would violate this bound. In particular, all `Client` instances will be operating
// synchronously.
unsafe impl<AccountId, Hash, Runtime: RuntimeT> Send
    for Client<AccountId, Hash, Runtime>
{
}
impl<AccountId, Hash, Runtime> Client<AccountId, Hash, Runtime>
where
    Runtime: RuntimeT + pallet_balances::Config + pallet_contracts::Config,
    AccountIdFor<Runtime>: From<[u8; 32]>,
    BalanceOf<Runtime>: From<u128>,
{
    pub fn new<P: Into<PathBuf>>(contracts: impl IntoIterator<Item = P>) -> Self {
        let mut sandbox = Sandbox::new().expect("Failed to initialize Drink! sandbox");
        Self::fund_accounts(&mut sandbox);

        Self {
            sandbox,
            contracts: ContractsRegistry::new(contracts),
            _phantom: Default::default(),
        }
    }

    fn fund_accounts(sandbox: &mut Sandbox<Runtime>) {
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
        for account in accounts.into_iter() {
            sandbox
                .mint_into(account, TOKENS.into())
                .unwrap_or_else(|_| panic!("Failed to mint {} tokens", TOKENS));
        }
    }
}

#[async_trait]
impl<AccountId: AsRef<[u8; 32]> + Send, Hash, Runtime> ChainBackend
    for Client<AccountId, Hash, Runtime>
where
    Runtime: RuntimeT + pallet_balances::Config,
    AccountIdFor<Runtime>: From<[u8; 32]>,
{
    type AccountId = AccountId;
    type Balance = BalanceOf<Runtime>;
    type Error = DrinkErr;
    type EventLog = ();

    async fn create_and_fund_account(
        &mut self,
        _origin: &Keypair,
        amount: Self::Balance,
    ) -> Keypair {
        let (pair, seed) = Pair::generate();

        self.sandbox
            .mint_into(pair.public().0.into(), amount)
            .expect("Failed to mint tokens");

        Keypair::from_seed(seed).expect("Failed to create keypair")
    }

    async fn free_balance(
        &mut self,
        account: Self::AccountId,
    ) -> Result<Self::Balance, Self::Error> {
        let account = AccountIdFor::<Runtime>::from(*account.as_ref());
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
        // perform some translation here in order to invoke strongly-typed drink!
        // API.

        // Get metadata of the drink! runtime, so that we can encode the call object.
        // Panic on error - metadata of the static im-memory runtime should always be
        // available.
        let raw_metadata: Vec<u8> = Runtime::get_metadata().into();
        let metadata = subxt_metadata::Metadata::decode(&mut raw_metadata.as_slice())
            .expect("Failed to decode metadata");

        // Encode the call object.
        let call = subxt::dynamic::tx(pallet_name, call_name, call_data);
        let encoded_call = call
            .encode_call_data(&metadata.into())
            .map_err(|_| DrinkErr)?;

        // Decode the call object.
        // Panic on error - we just encoded a validated call object, so it should be
        // decodable.
        let decoded_call = RuntimeCall::<Runtime>::decode(&mut encoded_call.as_slice())
            .expect("Failed to decode runtime call");

        // Execute the call.
        self.sandbox
            .runtime_call(
                decoded_call,
                Runtime::convert_account_to_origin(keypair_to_account(origin)),
            )
            .map_err(|_| DrinkErr)?;

        Ok(())
    }
}

#[async_trait]
impl<
        AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
        Hash: Copy + From<[u8; 32]>,
        Runtime: RuntimeT + pallet_balances::Config + pallet_contracts::Config,
        E: Environment<
                AccountId = AccountId,
                Balance = ContractsBalanceOf<Runtime>,
                Hash = Hash,
            > + 'static,
    > BuilderClient<E> for Client<AccountId, Hash, Runtime>
where
    AccountIdFor<Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
    ContractsBalanceOf<Runtime>: Send + Sync,
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
                return Err(DrinkErr) // todo: make a proper error type
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
        let result = self.sandbox.dry_run(|r| {
            r.deploy_contract(
                code,
                value,
                data,
                salt(),
                keypair_to_account(caller),
                DEFAULT_GAS_LIMIT,
                storage_deposit_limit,
            )
        });

        let account_id_raw = match &result.result {
            Err(_) => {
                panic!("Instantiate dry-run failed!")
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
                return Err(DrinkErr) // todo: make a proper error type
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

    async fn bare_call<Args: Sync + Encode + Clone, RetType: Send + Decode>(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        _gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<Self::EventLog, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        let account_id = message.clone().params().callee().clone();
        let exec_input = Encode::encode(message.clone().params().exec_input());
        let account_id = (*account_id.as_ref()).into();

        self.bare_call_dry_run(caller, message, value, storage_deposit_limit)
            .await?;

        if self
            .sandbox
            .call_contract(
                account_id,
                value,
                exec_input,
                keypair_to_account(caller),
                DEFAULT_GAS_LIMIT,
                storage_deposit_limit,
                pallet_contracts::Determinism::Enforced,
            )
            .result
            .is_err()
        {
            return Err(DrinkErr)
        }

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

        let result = self.sandbox.dry_run(|r| {
            r.call_contract(
                account_id,
                value,
                exec_input,
                keypair_to_account(caller),
                DEFAULT_GAS_LIMIT,
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
        Hash: Copy + From<[u8; 32]>,
        Runtime: RuntimeT + pallet_balances::Config + pallet_contracts::Config,
        E: Environment<
                AccountId = AccountId,
                Balance = ContractsBalanceOf<Runtime>,
                Hash = Hash,
            > + 'static,
    > E2EBackend<E> for Client<AccountId, Hash, Runtime>
where
    AccountIdFor<Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
    ContractsBalanceOf<Runtime>: Send + Sync,
{
}

fn keypair_to_account<AccountId: From<[u8; 32]>>(keypair: &Keypair) -> AccountId {
    AccountId::from(keypair.public_key().0)
}

#[async_trait]
impl<
        AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
        Hash: Copy + From<[u8; 32]>,
        Runtime: RuntimeT + pallet_balances::Config + pallet_contracts::Config,
        E: Environment<
                AccountId = AccountId,
                Balance = ContractsBalanceOf<Runtime>,
                Hash = Hash,
            > + 'static,
    > ContractsBackend<E> for Client<AccountId, Hash, Runtime>
where
    AccountIdFor<Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
{
    type Error = DrinkErr;
    type EventLog = ();
}
