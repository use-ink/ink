use crate::{
    builders::{
        constructor_exec_input,
        CreateBuilderPartial,
    },
    client_utils::{
        salt,
        ContractsRegistry,
    },
    log_error,
    CallBuilderFinal,
    CallDryRunResult,
    CallResult,
    ChainBackend,
    ContractsBackend,
    E2EBackend,
    InstantiationResult,
    UploadResult,
};
use drink::{
    chain_api::ChainApi,
    contract_api::ContractApi,
    runtime::{
        MinimalRuntime,
        Runtime,
    },
    Sandbox,
    DEFAULT_GAS_LIMIT,
};
use ink_env::Environment;
use jsonrpsee::core::async_trait;
use pallet_contracts_primitives::{
    CodeUploadReturnValue,
    ContractInstantiateResult,
    ContractResult,
    InstantiateReturnValue,
};
use scale::{
    Decode,
    Encode,
};
use sp_core::{
    crypto::AccountId32,
    sr25519::Pair,
    Pair as _,
};
use std::marker::PhantomData;
use subxt::dynamic::Value;
use subxt_signer::sr25519::Keypair;

pub struct Client<AccountId, Hash> {
    sandbox: Sandbox<MinimalRuntime>,
    contracts: ContractsRegistry,
    _phantom: PhantomData<(AccountId, Hash)>,
}

unsafe impl<AccountId, Hash> Send for Client<AccountId, Hash> {}

impl<AccountId, Hash> Client<AccountId, Hash> {
    pub fn new<'a>(contracts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut sandbox = Sandbox::new().expect("Failed to initialize Drink! sandbox");
        Self::fund_accounts(&mut sandbox);

        Self {
            sandbox,
            contracts: ContractsRegistry::new(contracts),
            _phantom: Default::default(),
        }
    }

    fn fund_accounts<R: Runtime>(sandbox: &mut Sandbox<R>) {
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
        .map(AccountId32::new);
        for account in accounts.into_iter() {
            sandbox.add_tokens(account, TOKENS);
        }
    }
}

#[async_trait]
impl<AccountId: AsRef<[u8; 32]> + Send, Hash> ChainBackend for Client<AccountId, Hash> {
    type AccountId = AccountId;
    type Balance = u128;
    type Error = ();
    type EventLog = ();

    async fn create_and_fund_account(
        &mut self,
        _origin: &Keypair,
        amount: Self::Balance,
    ) -> Keypair {
        let (pair, seed) = Pair::generate();

        self.sandbox.add_tokens(pair.public().0.into(), amount);

        Keypair::from_seed(seed).expect("Failed to create keypair")
    }

    async fn balance(
        &mut self,
        account: Self::AccountId,
    ) -> Result<Self::Balance, Self::Error> {
        let account = AccountId32::new(*account.as_ref());
        Ok(self.sandbox.balance(&account))
    }

    async fn runtime_call<'a>(
        &mut self,
        _origin: &Keypair,
        _pallet_name: &'a str,
        _call_name: &'a str,
        _call_data: Vec<Value>,
    ) -> Result<Self::EventLog, Self::Error> {
        todo!("https://github.com/Cardinal-Cryptography/drink/issues/36")
    }
}

#[async_trait]
impl<
        AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
        Hash: From<[u8; 32]>,
        E: Environment<AccountId = AccountId, Balance = u128, Hash = Hash> + 'static,
    > ContractsBackend<E> for Client<AccountId, Hash>
{
    type Error = ();
    type EventLog = ();

    async fn instantiate<Contract, Args: Send + Encode, R>(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<E, Self::EventLog>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        let data = constructor_exec_input(constructor);

        let result = self.sandbox.deploy_contract(
            code,
            value,
            data,
            salt(),
            keypair_to_account(caller),
            DEFAULT_GAS_LIMIT,
            storage_deposit_limit,
        );

        let account_id_raw = match &result.result {
            Err(err) => {
                log_error(&format!("Instantiation failed: {err:?}"));
                return Err(()) // todo: make a proper error type
            }
            Ok(res) => *res.account_id.as_ref(),
        };
        let account_id = AccountId::from(account_id_raw);

        Ok(InstantiationResult {
            account_id: account_id.clone(),
            // We need type remapping here because of the different `EventRecord` types.
            dry_run: ContractInstantiateResult {
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
            },
            events: (), // todo: https://github.com/Cardinal-Cryptography/drink/issues/32
        })
    }

    async fn instantiate_dry_run<Contract, Args: Send + Encode, R>(
        &mut self,
        _contract_name: &str,
        _caller: &Keypair,
        _constructor: CreateBuilderPartial<E, Contract, Args, R>,
        _value: E::Balance,
        _storage_deposit_limit: Option<E::Balance>,
    ) -> ContractInstantiateResult<E::AccountId, E::Balance, ()> {
        todo!("https://github.com/Cardinal-Cryptography/drink/issues/37")
    }

    async fn upload(
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
        ) {
            Ok(result) => result,
            Err(err) => {
                log_error(&format!("Upload failed: {err:?}"));
                return Err(()) // todo: make a proper error type
            }
        };

        Ok(UploadResult {
            code_hash: result.code_hash.0.into(),
            dry_run: Ok(CodeUploadReturnValue {
                code_hash: result.code_hash.0.into(),
                deposit: result.deposit,
            }),
            events: (),
        })
    }

    async fn call<Args: Sync + Encode, RetType: Send + Decode>(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<CallResult<E, RetType, Self::EventLog>, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        let account_id = message.clone().params().callee().clone();
        let exec_input = Encode::encode(message.clone().params().exec_input());
        let account_id = (*account_id.as_ref()).into();

        let result = self.sandbox.call_contract(
            account_id,
            value,
            exec_input,
            keypair_to_account(caller),
            DEFAULT_GAS_LIMIT,
            storage_deposit_limit,
        );

        Ok(CallResult {
            // We need type remapping here because of the different `EventRecord` types.
            dry_run: CallDryRunResult {
                exec_result: ContractResult {
                    gas_consumed: result.gas_consumed,
                    gas_required: result.gas_required,
                    storage_deposit: result.storage_deposit,
                    debug_message: result.debug_message,
                    result: result.result,
                    events: None,
                },
                _marker: Default::default(),
            },
            events: (), // todo: https://github.com/Cardinal-Cryptography/drink/issues/32
        })
    }

    async fn call_dry_run<Args: Sync + Encode, RetType: Send + Decode>(
        &mut self,
        _caller: &Keypair,
        _message: &CallBuilderFinal<E, Args, RetType>,
        _value: E::Balance,
        _storage_deposit_limit: Option<E::Balance>,
    ) -> CallDryRunResult<E, RetType>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        todo!("https://github.com/Cardinal-Cryptography/drink/issues/37")
    }
}

impl<
        AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
        Hash: From<[u8; 32]>,
        E: Environment<AccountId = AccountId, Balance = u128, Hash = Hash> + 'static,
    > E2EBackend<E> for Client<AccountId, Hash>
{
}

fn keypair_to_account(keypair: &Keypair) -> AccountId32 {
    AccountId32::from(keypair.public_key().0)
}
