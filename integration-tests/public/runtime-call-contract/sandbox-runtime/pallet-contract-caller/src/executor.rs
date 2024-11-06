use crate::{
    AccountIdOf,
    BalanceOf,
};
use frame_support::pallet_prelude::Weight;
use ink::env::{
    call::{
        ExecutionInput,
        Executor,
    },
    Environment,
};

pub struct PalletContractsExecutor<E: Environment, Runtime: pallet_contracts::Config> {
    pub origin: AccountIdOf<Runtime>,
    pub contract: AccountIdOf<Runtime>,
    pub value: BalanceOf<Runtime>,
    pub gas_limit: Weight,
    pub storage_deposit_limit: Option<BalanceOf<Runtime>>,
    pub marker: core::marker::PhantomData<E>,
}

impl<E, R> Executor<E> for PalletContractsExecutor<E, R>
where
    E: Environment,
    R: pallet_contracts::Config,
{
    type Error = sp_runtime::DispatchError;

    fn exec<Args, Output>(
        &self,
        input: &ExecutionInput<Args>,
    ) -> Result<ink::MessageResult<Output>, Self::Error>
    where
        Args: codec::Encode,
        Output: codec::Decode,
    {
        let data = codec::Encode::encode(&input);

        let result = pallet_contracts::Pallet::<R>::bare_call(
            self.origin.clone(),
            self.contract.clone(),
            self.value,
            self.gas_limit,
            self.storage_deposit_limit,
            data,
            pallet_contracts::DebugInfo::UnsafeDebug,
            pallet_contracts::CollectEvents::Skip,
            pallet_contracts::Determinism::Enforced,
        );

        let output = result.result?.data;
        let result = codec::Decode::decode(&mut &output[..]).map_err(|_| {
            sp_runtime::DispatchError::Other("Failed to decode contract output")
        })?;

        Ok(result)
    }
}
