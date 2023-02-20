use crate::{
    builders::CreateBuilderPartial, client, log_info, Client, Error, InstantiationResult,
    Signer,
};
use async_trait::async_trait;
use ink::ConstructorCallable;
use ink_env::Environment;
use sp_core::sr25519;
use std::fmt::Debug;
use subxt::config::ExtrinsicParams;

#[async_trait]
pub trait DispatchInstantiate<C, E, ContractRef, Args, R>
where
    Args: scale::Encode,
    C: subxt::Config,
    C::AccountId: From<sp_runtime::AccountId32>
        + scale::Codec
        + serde::de::DeserializeOwned
        + Debug,
    C::Signature: From<sr25519::Signature>,
    <C::ExtrinsicParams as ExtrinsicParams<C::Index, C::Hash>>::OtherParams: Default,

    E: Environment,
    E::AccountId: Debug,
    E::Balance: Debug + scale::HasCompact + serde::Serialize,
    E::Hash: Debug + scale::Encode,
{
    async fn instantiate(
        self,
        contract_name: &str,
        signer: &Signer<C>,
        client: &mut Client<C, E>,
    ) -> Result<InstantiationResult<C, E>, Error<C, E>>;
}

#[async_trait]
impl<ContractRef, Args, R, E, C> DispatchInstantiate<C, E, ContractRef, Args, R>
    for ConstructorCallable<ContractRef, Args, R, E>
where
    Args: scale::Encode + Send + Sync,
    C: subxt::Config + Send + Sync,
    C::AccountId: From<sp_runtime::AccountId32>
        + scale::Codec
        + serde::de::DeserializeOwned
        + Debug
        + Send
        + Sync,
    C::Signature: From<sr25519::Signature>,
    <C::ExtrinsicParams as ExtrinsicParams<C::Index, C::Hash>>::OtherParams:
        Default + Send + Sync,

    E: Environment + Send + Sync,
    E::AccountId: Debug + Send + Sync,
    E::Balance: Debug + scale::HasCompact + serde::Serialize + Send + Sync,
    E::Hash: Debug + scale::Encode + Send + Sync,
{
    async fn instantiate(
        self,
        contract_name: &str,
        signer: &Signer<C>,
        client: &mut Client<C, E>,
    ) -> Result<InstantiationResult<C, E>, Error<C, E>> {
        let contracts = client.contracts();
        let contract_metadata = contracts
            .get(contract_name)
            .ok_or_else(|| Error::ContractNotFound(contract_name.to_owned()))?;
        let code = crate::utils::extract_wasm(contract_metadata);
        let value = self.value();
        let storage_deposit_limit = self.storage_deposit_limit();
        let constructor = self.constructor();
        let ret = client
            .exec_instantiate(signer, code, constructor, value, storage_deposit_limit)
            .await?;
        log_info(&format!("instantiated contract at {:?}", ret.account_id));
        Ok(ret)
    }
}
