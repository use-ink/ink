use crate::{
    builders::{
        constructor_exec_input,
        CreateBuilderPartial,
    },
    log_error,
    log_info,
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
    runtime::MinimalRuntime,
    session::Session,
    DEFAULT_GAS_LIMIT,
};
use ink_env::Environment;
use jsonrpsee::core::async_trait;
use pallet_contracts_primitives::{
    ContractInstantiateResult,
    ContractResult,
};
use scale::{
    Decode,
    Encode,
};
use sp_core::{
    crypto::AccountId32,
    sr25519::Pair,
    Pair as _,
    H256,
};
use std::{
    collections::BTreeMap,
    path::PathBuf,
};
use subxt::dynamic::Value;
use subxt_signer::sr25519::Keypair;

pub struct Client {
    session: Session<MinimalRuntime>,
    contracts: BTreeMap<String, PathBuf>,
}

unsafe impl Send for Client {}

impl Client {
    pub fn new<'a>(contracts: impl IntoIterator<Item = &'a str>) -> Self {
        // todo: extract to a common place
        let contracts = contracts
            .into_iter()
            .map(|path| {
                let wasm_path = PathBuf::from(path);
                let contract_name = wasm_path.file_stem().unwrap_or_else(|| {
                    panic!("Invalid contract wasm path '{}'", wasm_path.display(),)
                });
                (contract_name.to_string_lossy().to_string(), wasm_path)
            })
            .collect();

        Self {
            session: Session::new(None).expect("Failed to create session"),
            contracts,
        }
    }

    // todo: extract to a common place
    fn load_code(&self, contract: &str) -> Vec<u8> {
        let wasm_path = self
            .contracts
            .get(&contract.replace('-', "_"))
            .unwrap_or_else(||
                panic!(
                    "Unknown contract {contract}. Available contracts: {:?}.\n\
                     For a contract to be built, add it as a dependency to the `Cargo.toml`, or add \
                     the manifest path to `#[ink_e2e::test(additional_contracts = ..)]`",
                    self.contracts.keys()
                )
            );
        let code = std::fs::read(wasm_path).unwrap_or_else(|err| {
            panic!("Error loading '{}': {:?}", wasm_path.display(), err)
        });
        log_info(&format!("{:?} has {} KiB", contract, code.len() / 1024));
        code
    }

    // todo: extract to a common place
    fn salt() -> Vec<u8> {
        use funty::Fundamental as _;

        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|err| panic!("unable to get unix time: {err}"))
            .as_millis()
            .as_u128()
            .to_le_bytes()
            .to_vec()
    }
}

#[async_trait]
impl ChainBackend for Client {
    type AccountId = AccountId32;
    type Balance = u128;
    type Error = ();
    type EventLog = ();

    async fn create_and_fund_account(
        &mut self,
        _origin: &Keypair,
        amount: Self::Balance,
    ) -> Keypair {
        // todo: extract to a common place
        let (pair, seed) = Pair::generate();

        self.session
            .chain_api()
            .add_tokens(pair.public().0.into(), amount);

        Keypair::from_seed(seed).expect("Failed to create keypair")
    }

    async fn balance(
        &mut self,
        actor: Self::AccountId,
    ) -> Result<Self::Balance, Self::Error> {
        Ok(self.session.chain_api().balance(&actor))
    }

    async fn runtime_call<'a>(
        &mut self,
        _actor: &Keypair,
        _pallet_name: &'a str,
        _call_name: &'a str,
        _call_data: Vec<Value>,
    ) -> Result<Self::EventLog, Self::Error> {
        todo!("https://github.com/Cardinal-Cryptography/drink/issues/36")
    }
}

#[async_trait]
impl<E: Environment<AccountId = AccountId32, Balance = u128, Hash = H256> + 'static>
    ContractsBackend<E> for Client
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
        let code = self.load_code(contract_name);
        let data = constructor_exec_input(constructor);

        let result = self.session.contracts_api().deploy_contract(
            code,
            value,
            data,
            Self::salt(),
            keypair_to_account(caller),
            DEFAULT_GAS_LIMIT,
            storage_deposit_limit,
        );
        let account_id = self
            .session
            .last_deploy_return()
            .expect("We have just deployed a contract, so we should have its address");

        Ok(InstantiationResult {
            account_id,
            // We need type remapping here because of the different `EventRecord` types.
            dry_run: ContractInstantiateResult {
                gas_consumed: result.gas_consumed,
                gas_required: result.gas_required,
                storage_deposit: result.storage_deposit,
                debug_message: result.debug_message,
                result: result.result,
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
        let code = self.load_code(contract_name);

        let result = match self.session.contracts_api().upload_contract(
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
            code_hash: result.code_hash,
            dry_run: Ok(result),
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

        let result = self.session.contracts_api().call_contract(
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

impl<E: Environment<AccountId = AccountId32, Balance = u128, Hash = H256> + 'static>
    E2EBackend<E> for Client
{
}

fn keypair_to_account(keypair: &Keypair) -> AccountId32 {
    AccountId32::from(keypair.public_key().0)
}
