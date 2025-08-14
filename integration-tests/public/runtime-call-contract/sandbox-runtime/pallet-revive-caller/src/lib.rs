//! # Contract Caller
//!
//! Demonstrates calling into an `ink!` contract from a pallet.

#![cfg_attr(not(feature = "std"), no_std)]

mod executor;

use frame_support::{
    pallet_prelude::Weight,
    traits::fungible::Inspect,
};
pub use pallet::*;

//type AccountIdOf<R> = <R as frame_system::Config>::AccountId;
type BalanceOf<R> = <<R as pallet_revive::Config>::Currency as Inspect<
    <R as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use flipper_traits::Flip;
    use frame_support::{
        pallet_prelude::*,
        traits::fungible::Inspect,
    };
    use frame_system::pallet_prelude::*;
    use pallet_revive::{
        evm::*,
        MomentOf,
    };
    use sp_runtime::traits::Bounded;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_revive::Config {}

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
        <<T as pallet_revive::Config>::Currency as Inspect<
            <T as frame_system::Config>::AccountId,
        >>::Balance: From<u128>,

        BalanceOf<T>: Into<U256> + TryFrom<U256> + Bounded,
        MomentOf<T>: Into<U256>,
        <T as frame_system::Config>::Hash: IsType<sp_runtime::testing::H256>,
    {
        /// Call the flip method on the contract at the given `contract` account.
        #[pallet::call_index(0)]
        #[pallet::weight(<T::WeightInfo as pallet_revive::WeightInfo>::call().saturating_add(*gas_limit))]
        pub fn contract_call_flip(
            origin: OriginFor<T>,
            contract: H160,
            gas_limit: Weight,
        ) -> DispatchResult {
            let _who = ensure_signed(origin.clone())?;

            let executor =
                executor::PalletReviveExecutor::<ink::env::DefaultEnvironment, T> {
                    origin: origin.clone(),
                    contract,
                    value: 0.into(),
                    gas_limit,
                    marker: Default::default(),
                };

            let mut flipper = ink::message_builder!(Flip);
            let result = flipper.flip().exec(&executor)?;

            assert!(result.is_ok());

            Ok(())
        }
    }
}
