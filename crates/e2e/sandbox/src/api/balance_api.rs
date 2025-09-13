use crate::{
    AccountIdFor,
    OriginFor,
    Sandbox,
};
use frame_support::{
    sp_runtime::DispatchError,
    traits::fungible::Mutate,
};
use pallet_revive::sp_runtime::traits::StaticLookup;

type BalanceOf<R> = <R as pallet_balances::Config>::Balance;

/// Balance API for the sandbox.
pub trait BalanceAPI<T: Sandbox>
where
    T: Sandbox,
    T::Runtime: pallet_balances::Config,
{
    /// Mint tokens to an account.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the account to add tokens to.
    /// * `amount` - The number of tokens to add.
    fn mint_into(
        &mut self,
        address: &AccountIdFor<T::Runtime>,
        amount: BalanceOf<T::Runtime>,
    ) -> Result<BalanceOf<T::Runtime>, DispatchError>;

    /// Return the free balance of an account.
    ///
    /// # Arguments
    ///
    /// * `account` - The account id of the account to query.
    fn free_balance(
        &mut self,
        account_id: &AccountIdFor<T::Runtime>,
    ) -> BalanceOf<T::Runtime>;

    fn transfer_allow_death(
        &mut self,
        origin: &OriginFor<T::Runtime>,
        dest: &AccountIdFor<T::Runtime>,
        value: BalanceOf<T::Runtime>,
    ) -> Result<(), DispatchError>;
}

impl<T> BalanceAPI<T> for T
where
    T: Sandbox,
    T::Runtime: pallet_balances::Config,
{
    fn mint_into(
        &mut self,
        address: &AccountIdFor<T::Runtime>,
        amount: BalanceOf<T::Runtime>,
    ) -> Result<BalanceOf<T::Runtime>, DispatchError> {
        self.execute_with(|| {
            pallet_balances::Pallet::<T::Runtime>::mint_into(address, amount)
        })
    }

    fn free_balance(
        &mut self,
        account_id: &AccountIdFor<T::Runtime>,
    ) -> BalanceOf<T::Runtime> {
        self.execute_with(|| {
            pallet_balances::Pallet::<T::Runtime>::free_balance(account_id)
        })
    }

    fn transfer_allow_death(
        &mut self,
        origin: &OriginFor<T::Runtime>,
        dest: &AccountIdFor<T::Runtime>,
        value: BalanceOf<T::Runtime>,
    ) -> Result<(), DispatchError> {
        // Convert AccountId into the proper `Lookup::Source`
        let dest =
            <<T::Runtime as frame_system::Config>::Lookup as StaticLookup>::unlookup(
                dest.clone(),
            );

        self.execute_with(|| {
            pallet_balances::Pallet::<T::Runtime>::transfer_allow_death(
                origin.clone(),
                dest,
                value,
            )
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::DefaultSandbox;
    #[test]
    fn mint_works() {
        let mut sandbox = DefaultSandbox::default();
        let balance = sandbox.free_balance(&DefaultSandbox::default_actor());

        sandbox
            .mint_into(&DefaultSandbox::default_actor(), 100)
            .unwrap();

        assert_eq!(
            sandbox.free_balance(&DefaultSandbox::default_actor()),
            balance + 100
        );
    }
}
