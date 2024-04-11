//! # Contract Caller
//!
//! Demonstrates calling into an `ink!` contract from a pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::Weight;
use frame_support::traits::fungible::Inspect;
use ink::env::call::{ExecutionInput, Invoke, Invoker};
pub use pallet::*;
use ink::env::Environment;

type AccountIdOf<R> = <R as frame_system::Config>::AccountId;
type BalanceOf<R> = <<R as pallet_contracts::Config>::Currency as Inspect<<R as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use flipper_traits::Flip;
    use frame_support::{
        pallet_prelude::*,
        traits::fungible::Inspect,
    };
    use frame_system::pallet_prelude::*;
    use ink::codegen::TraitCallBuilder;
    use ink::env::call::utils::EmptyArgumentList;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_contracts::Config {}

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
        <<T as pallet_contracts::Config>::Currency as Inspect<
            <T as frame_system::Config>::AccountId,
        >>::Balance: From<u128>,
    {
        /// Call the flip method on the contract at the given `contract` account.
        #[pallet::call_index(0)]
        #[pallet::weight(<T::WeightInfo as pallet_contracts::WeightInfo>::call().saturating_add(*gas_limit))]
        pub fn contract_call_flip(
            origin: OriginFor<T>,
            contract: AccountIdOf<T>,
            gas_limit: Weight,
            storage_deposit_limit: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let ink_account_id =
                ink::primitives::AccountId::from(<[u8; 32]>::from(contract.clone()));
            let mut flipper: ink::contract_ref!(Flip, ink::env::DefaultEnvironment) =
                ink_account_id.into();
            let call_builder = flipper.call_mut();

            let invoker = PalletContractsInvoker::<ink::env::DefaultEnvironment, T> {
                origin: who.clone(),
                contract: contract.clone(),
                value: 0.into(),
                gas_limit,
                storage_deposit_limit,
                marker: Default::default()
            };

            let call = call_builder.flip().params();

            let invoke = Invoke::<EmptyArgumentList, ()>::new(call.exec_input().clone());

            let result = invoke.invoke(invoker).unwrap();

            assert!(result.is_ok());

            Ok(())
        }
    }
}

struct PalletContractsInvoker<E: Environment, Runtime: pallet_contracts::Config> {
    origin: AccountIdOf<Runtime>,
    contract: AccountIdOf<Runtime>,
    value: BalanceOf<Runtime>,
    gas_limit: Weight,
    storage_deposit_limit: Option<BalanceOf<Runtime>>,
    marker: core::marker::PhantomData<E>,
}

impl<E, R> Invoker<E> for PalletContractsInvoker<E, R>
where
    E: Environment,
    R: pallet_contracts::Config,
{
    fn invoke<Args, Output>(self, input: &ExecutionInput<Args>) -> Result<ink::MessageResult<Output>, ()>
    where
        Args: codec::Encode,
        Output: codec::Decode,
    {
        let data = codec::Encode::encode(&input);

        let result = pallet_contracts::Pallet::<R>::bare_call(
            self.origin,
            self.contract,
            self.value,
            self.gas_limit,
            self.storage_deposit_limit,
            data,
            pallet_contracts::DebugInfo::UnsafeDebug,
            pallet_contracts::CollectEvents::Skip,
            pallet_contracts::Determinism::Enforced,
        );

        let output = result.result.unwrap().data;

        Ok(codec::Decode::decode(&mut &output[..]).unwrap())
    }
}

