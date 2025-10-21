use crate::{
    AccountIdFor,
    Sandbox,
};
use frame_support::{
    pallet_prelude::DispatchError,
    traits::fungibles::{
        Create,
        Destroy,
        Inspect,
        Mutate,
        approvals::{
            Inspect as _,
            Mutate as _,
        },
        metadata::Mutate as _,
    },
};

type AssetIdOf<T, I> = <T as pallet_assets::Config<I>>::AssetId;
type AssetBalanceOf<T, I> = <T as pallet_assets::Config<I>>::Balance;

/// Assets API for the sandbox.
///
/// Provides methods to create, mint, and manage assets in `pallet-assets`.
pub trait AssetsAPI<T, I = pallet_assets::Instance1>
where
    T: Sandbox,
    T::Runtime: pallet_assets::Config<I>,
    I: 'static,
{
    /// Creates `value` amount of tokens and assigns them to `account`, increasing the
    /// total supply.
    ///
    /// # Arguments
    /// * `id` - ID of the new asset to be created.
    /// * `owner` - The owner of the created asset.
    /// * `min_balance` - The asset amount one account need at least.
    fn create(
        &mut self,
        id: &AssetIdOf<T::Runtime, I>,
        owner: &AccountIdFor<T::Runtime>,
        min_balance: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError>;

    /// Start the destruction an existing fungible asset.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    fn start_destroy(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
    ) -> Result<(), DispatchError>;

    /// Start the destruction an existing fungible asset.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `name` - Token name.
    /// * `symbol` - Token symbol.
    /// * `decimals` - Token decimals.
    fn set_metadata(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: &AccountIdFor<T::Runtime>,
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
    ) -> Result<(), DispatchError>;

    /// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
    ///
    /// Successive calls of this method overwrite previous values.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `spender` - The account that is allowed to spend the tokens.
    /// * `value` - The number of tokens to approve.
    fn approve(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: &AccountIdFor<T::Runtime>,
        delegate: &AccountIdFor<T::Runtime>,
        amount: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError>;

    /// Creates `value` amount of tokens and assigns them to `account`, increasing the
    /// total supply.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `account` - The account to be credited with the created tokens.
    /// * `value` - The number of tokens to mint.
    fn mint_into(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        account: &AccountIdFor<T::Runtime>,
        value: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<AssetBalanceOf<T::Runtime, I>, DispatchError>;

    /// Transfer `amount` of tokens from `origin` to `dest`.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `source` - The account from which tokens are transferred.
    /// * `dest` - The account to which tokens are transferred.
    /// * `amount` - The number of tokens to transfer.
    fn transfer(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        source: &AccountIdFor<T::Runtime>,
        dest: &AccountIdFor<T::Runtime>,
        amount: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError>;

    /// Returns the account balance for the specified `owner`.
    ///
    /// # Arguments
    /// * `owner` - The account whose balance is being queried.
    fn balance_of(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: &AccountIdFor<T::Runtime>,
    ) -> AssetBalanceOf<T::Runtime, I>;

    /// Returns the total supply of the `asset`.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    fn total_supply(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
    ) -> AssetBalanceOf<T::Runtime, I>;

    /// Returns the allowance for a `spender` approved by an `owner`.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `owner` - The account that owns the tokens.
    /// * `spender` - The account that is allowed to spend the tokens.
    fn allowance(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: &AccountIdFor<T::Runtime>,
        delegate: &AccountIdFor<T::Runtime>,
    ) -> AssetBalanceOf<T::Runtime, I>;

    /// Check if the asset exists.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    fn asset_exists(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> bool;
}

impl<T, I> AssetsAPI<T, I> for T
where
    T: Sandbox,
    T::Runtime: pallet_assets::Config<I>,
    I: 'static,
{
    fn create(
        &mut self,
        id: &AssetIdOf<T::Runtime, I>,
        owner: &AccountIdFor<T::Runtime>,
        min_balance: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError> {
        self.execute_with(|| {
            <pallet_assets::Pallet<T::Runtime, I> as Create<
                AccountIdFor<T::Runtime>,
            >>::create(id.clone(), owner.clone(), true, min_balance)
        })
    }

    fn start_destroy(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
    ) -> Result<(), DispatchError> {
        self.execute_with(|| {
            <pallet_assets::Pallet<T::Runtime, I> as Destroy<
                AccountIdFor<T::Runtime>,
            >>::start_destroy(asset.clone(), None)
        })
    }

    fn set_metadata(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: &AccountIdFor<T::Runtime>,
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
    ) -> Result<(), DispatchError> {
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::set(
                asset.clone().into(),
                owner,
                name,
                symbol,
                decimals,
            )
        })
    }

    fn mint_into(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        account: &AccountIdFor<T::Runtime>,
        value: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<AssetBalanceOf<T::Runtime, I>, DispatchError> {
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::mint_into(
                asset.clone(),
                account,
                value,
            )
        })
    }

    fn transfer(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        source: &AccountIdFor<T::Runtime>,
        dest: &AccountIdFor<T::Runtime>,
        amount: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError> {
        self.execute_with(|| {
            <pallet_assets::Pallet<T::Runtime, I> as Mutate<AccountIdFor<T::Runtime>>>::transfer(
                asset.clone(),
                source,
                dest,
                amount,
                frame_support::traits::tokens::Preservation::Preserve,
            ).map(|_| ())
        })
    }

    fn approve(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: &AccountIdFor<T::Runtime>,
        delegate: &AccountIdFor<T::Runtime>,
        amount: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError> {
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::approve(
                asset.clone(),
                owner,
                delegate,
                amount,
            )
        })
    }

    fn balance_of(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: &AccountIdFor<T::Runtime>,
    ) -> AssetBalanceOf<T::Runtime, I> {
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::balance(asset.clone(), owner)
        })
    }

    fn total_supply(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
    ) -> AssetBalanceOf<T::Runtime, I> {
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::total_supply(asset.clone())
        })
    }

    fn allowance(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: &AccountIdFor<T::Runtime>,
        delegate: &AccountIdFor<T::Runtime>,
    ) -> AssetBalanceOf<T::Runtime, I> {
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::allowance(
                asset.clone(),
                owner,
                delegate,
            )
        })
    }

    fn asset_exists(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> bool {
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::asset_exists(asset.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DefaultSandbox;

    #[test]
    fn create_works() {
        let mut sandbox = DefaultSandbox::default();
        let admin = DefaultSandbox::default_actor();
        let asset_id = 1u32;
        let min_balance = 1u128;

        let result = sandbox.create(&asset_id, &admin, min_balance);

        assert!(result.is_ok());
        assert!(sandbox.asset_exists(&asset_id));
    }

    #[test]
    fn mint_asset_works() {
        let mut sandbox = DefaultSandbox::default();
        let admin = DefaultSandbox::default_actor();
        let asset_id = 1u32;

        sandbox.create(&asset_id, &admin, 1u128).unwrap();

        let balance_before = sandbox.balance_of(&asset_id, &admin);
        assert_eq!(balance_before, 0);

        sandbox.mint_into(&asset_id, &admin, 100u128).unwrap();

        let balance_after = sandbox.balance_of(&asset_id, &admin);
        assert_eq!(balance_after, 100);
    }

    #[test]
    fn total_supply_works() {
        let mut sandbox = DefaultSandbox::default();
        let admin = DefaultSandbox::default_actor();
        let asset_id = 1u32;

        sandbox.create(&asset_id, &admin, 1u128).unwrap();

        let supply_before = sandbox.total_supply(&asset_id);
        assert_eq!(supply_before, 0);

        sandbox.mint_into(&asset_id, &admin, 1000u128).unwrap();

        let supply_after = sandbox.total_supply(&asset_id);
        assert_eq!(supply_after, 1000);
    }

    #[test]
    fn asset_exists_works() {
        let mut sandbox = DefaultSandbox::default();
        let admin = DefaultSandbox::default_actor();
        let asset_id = 1u32;

        assert!(!sandbox.asset_exists(&asset_id));

        sandbox.create(&asset_id, &admin, 1u128).unwrap();

        assert!(sandbox.asset_exists(&asset_id));
    }
}
