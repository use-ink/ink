use crate::{
    BalanceOf,
};
use frame_support::pallet_prelude::Weight;
use frame_support::traits::IsType;
use frame_system::pallet_prelude::OriginFor;
use pallet_revive::{DepositLimit, MomentOf};
use sp_runtime::traits::Bounded;
use ink::env::{
    call::{
        ExecutionInput,
        Executor,
    },
    Environment,
};
use ink::H160;
use ink::primitives::U256;

pub struct PalletReviveExecutor<E: Environment, Runtime: pallet_revive::Config> {
    //pub origin: AccountIdOf<Runtime>,
    pub origin: OriginFor<Runtime>,
    pub contract: H160,
    pub value: BalanceOf<Runtime>,
    pub gas_limit: Weight,
    //pub storage_deposit_limit: Option<BalanceOf<Runtime>>,
    //pub storage_deposit_limit: u128,
    pub marker: core::marker::PhantomData<E>,
}

impl<E, R> Executor<E> for PalletReviveExecutor<E, R>
where
    E: Environment,
    R: pallet_revive::Config,

    BalanceOf<R>: Into<U256> + TryFrom<U256> + Bounded,
    MomentOf<R>: Into<U256>,
    <R as frame_system::Config>::Hash: IsType<sp_runtime::testing::H256>
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

        let result = pallet_revive::Pallet::<R>::bare_call(
            self.origin.clone(),
            // <R as pallet_revive::Config>::AddressMapper::to_account_id(&self.contract),
            self.contract,
            self.value,
            self.gas_limit,
            // self.storage_deposit_limit,
            DepositLimit::Unchecked, // todo
            data,
            pallet_revive::DebugInfo::UnsafeDebug,
            pallet_revive::CollectEvents::Skip,
        );

        let output = result.result?.data;
        let result = codec::Decode::decode(&mut &output[..]).map_err(|_| {
            sp_runtime::DispatchError::Other("Failed to decode contract output")
        })?;

        Ok(result)
    }
}
