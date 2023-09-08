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
    chain_api::{
        ChainApi,
        RuntimeCall,
    },
    contract_api::ContractApi,
    runtime::Runtime as RuntimeT,
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

pub struct Client<AccountId, Hash, Runtime: RuntimeT> {
    sandbox: Sandbox<Runtime>,
    contracts: ContractsRegistry,
    _phantom: PhantomData<(AccountId, Hash)>,
}

// While it is not necessary true that `Client` is `Send`, it will not be used in a way that would
// violate this bound. In particular, all `Client` instances will be operating synchronously.
unsafe impl<AccountId, Hash, Runtime: RuntimeT> Send
    for Client<AccountId, Hash, Runtime>
{
}

type RuntimeAccountId<R> = <R as drink::runtime::frame_system::Config>::AccountId;

impl<AccountId, Hash, Runtime: RuntimeT> Client<AccountId, Hash, Runtime>
where
    RuntimeAccountId<Runtime>: From<[u8; 32]>,
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
            sandbox.add_tokens(account, TOKENS);
        }
    }
}

#[async_trait]
impl<AccountId: AsRef<[u8; 32]> + Send, Hash, Runtime: RuntimeT> ChainBackend
    for Client<AccountId, Hash, Runtime>
where
    RuntimeAccountId<Runtime>: From<[u8; 32]>,
{
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
        let account = RuntimeAccountId::<Runtime>::from(*account.as_ref());
        Ok(self.sandbox.balance(&account))
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
        let encoded_call = call.encode_call_data(&metadata.into()).map_err(|_| ())?;

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
            .map_err(|_| ())?;

        Ok(())
    }
}

#[async_trait]
impl<
        AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
        Hash: Copy + From<[u8; 32]>,
        Runtime: RuntimeT,
        E: Environment<AccountId = AccountId, Balance = u128, Hash = Hash> + 'static,
    > ContractsBackend<E> for Client<AccountId, Hash, Runtime>
where
    RuntimeAccountId<Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
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
        Hash: Copy + From<[u8; 32]>,
        Runtime: RuntimeT,
        E: Environment<AccountId = AccountId, Balance = u128, Hash = Hash> + 'static,
    > E2EBackend<E> for Client<AccountId, Hash, Runtime>
where
    RuntimeAccountId<Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
{
}

fn keypair_to_account<AccountId: From<[u8; 32]>>(keypair: &Keypair) -> AccountId {
    AccountId::from(keypair.public_key().0)
}
