use ink_env::{call::CreateParams, Environment};
use scale::Encode;

use super::builders::{CreateBuilderPartial, finalise_constructor};



pub struct ConstructorCallable<ContractRef, Args, R, E>
where
    E: Environment,
{
    constructor: CreateBuilderPartial<E, ContractRef, Args, R>,
    value: E::Balance,
    storage_deposit_limit: Option<E::Balance>,
}


impl<ContractRef, Args, R, E> ConstructorCallable<ContractRef, Args, R, E>
where
    E: Environment,
{
    pub fn new(
        constructor: CreateBuilderPartial<E, ContractRef, Args, R>,
    ) -> Self
    where
        Args: Encode,
    {
        Self {
            constructor,
            value: 0u8.into(),
            storage_deposit_limit: None,
        }
    }

    pub fn with_value(&mut self, value: E::Balance) {
        self.value = value;
    }

    pub fn with_storage_deposit_limit(&mut self, storage_deposit_limit: E::Balance) {
        self.storage_deposit_limit = Some(storage_deposit_limit);
    }

    pub fn value(&self) -> E::Balance {
        self.value
    }

    pub fn storage_deposit_limit(&self) -> Option<E::Balance> {
        self.storage_deposit_limit
    }

    pub fn constructor(self) -> CreateBuilderPartial<E, ContractRef, Args, R> {
        self.constructor
    }



    //TODO: potentially supply `contracts` and `api` here
    // pub async fn call(
    //     mut self,
    //     contract_name: &str,
    //     signer: &Signer<C>,
    // ) -> Result<InstantiationResult<C, E>, Error<C, E>>
    // where
    //     Args: scale::Encode,
    // {
    //     let contract_metadata = self
    //         .contracts
    //         .get(contract_name)
    //         .ok_or_else(|| Error::ContractNotFound(contract_name.to_owned()))?;
    //     let code = super::utils::extract_wasm(contract_metadata);
    //     let ret = self.exec_instantiate(signer, code).await?;
    //     log_info(&format!("instantiated contract at {:?}", ret.account_id));
    //     Ok(ret)
    // }
}
