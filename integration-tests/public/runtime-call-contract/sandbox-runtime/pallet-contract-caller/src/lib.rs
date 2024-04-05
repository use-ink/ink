//! # Contract Caller
//!
//! Demonstrates calling into an `ink!` contract from a pallet.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

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

    type AccountIdOf<Runtime> = <Runtime as frame_system::Config>::AccountId;

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
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let ink_account_id =
                ink::primitives::AccountId::from(<[u8; 32]>::from(contract.clone()));
            let mut flipper: ink::contract_ref!(Flip, ink::env::DefaultEnvironment) =
                ink_account_id.into();
            let call_builder = flipper.call_mut();

            let params = call_builder
                .flip()
                .ref_time_limit(gas_limit.ref_time())
                .proof_size_limit(gas_limit.proof_size())
                .params();

            // Next step is to explore ways to encapsulate the following into the call
            // builder.
            let value = *params.transferred_value();
            let data = params.exec_input().encode();
            let weight =
                Weight::from_parts(params.ref_time_limit(), params.proof_size_limit());
            let storage_deposit_limit =
                params.storage_deposit_limit().map(|limit| (*limit).into());

            let result = pallet_contracts::Pallet::<T>::bare_call(
                who.clone(),
                contract.clone(),
                value.into(),
                weight,
                storage_deposit_limit,
                data,
                pallet_contracts::DebugInfo::UnsafeDebug,
                pallet_contracts::CollectEvents::Skip,
                pallet_contracts::Determinism::Enforced,
            );

            println!("Flip result: {:?}", result);

            assert!(!result.result?.did_revert());

            Ok(())
        }
    }
}
