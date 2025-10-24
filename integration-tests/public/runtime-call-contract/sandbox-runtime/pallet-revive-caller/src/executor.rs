use crate::BalanceOf;
use frame_support::{
    pallet_prelude::Weight,
    traits::IsType,
};
use frame_system::pallet_prelude::OriginFor;
use ink::{
    Address,
    MessageResult,
    env::{
        Environment,
        call::{
            ExecutionInput,
            Executor,
            utils::{
                DecodeMessageResult,
                EncodeArgsWith,
            },
        },
    },
    primitives::U256,
};
use pallet_revive::{
    ExecConfig,
    MomentOf,
};
use sp_runtime::traits::Bounded;

pub struct PalletReviveExecutor<E: Environment, Runtime: pallet_revive::Config> {
    pub origin: OriginFor<Runtime>,
    pub contract: Address,
    pub value: BalanceOf<Runtime>,
    pub gas_limit: Weight,
    // todo
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
    <R as frame_system::Config>::Hash: IsType<sp_runtime::testing::H256>,
{
    type Error = sp_runtime::DispatchError;

    fn exec<Args, Output, Abi>(
        &self,
        input: &ExecutionInput<Args, Abi>,
    ) -> Result<MessageResult<Output>, Self::Error>
    where
        Args: EncodeArgsWith<Abi>,
        Output: DecodeMessageResult<Abi>,
    {
        let data = input.encode();

        let result = pallet_revive::Pallet::<R>::bare_call(
            self.origin.clone(),
            self.contract,
            ink_sandbox::balance_to_evm_value::<R>(self.value),
            self.gas_limit,
            // self.storage_deposit_limit,
            BalanceOf::<R>::max_value(), // todo
            data,
            ExecConfig {
                bump_nonce: true,
                collect_deposit_from_hold: None,
                effective_gas_price: None,
            },
        );

        let output = result.result?;
        let result =
            DecodeMessageResult::decode_output(&output.data[..], output.did_revert())
                .map_err(|_| {
                    sp_runtime::DispatchError::Other("Failed to decode contract output")
                })?;

        Ok(result)
    }
}
