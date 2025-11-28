use crate::{
    AccountIdFor,
    IntoAccountId,
    RuntimeEnv,
};
use frame_support::{
    pallet_prelude::DispatchError,
    traits::fungibles::{
        Create,
        Inspect,
        Mutate,
        approvals::{
            Inspect as _,
            Mutate as _,
        },
        metadata::{
            Inspect as MetadataInspect,
            Mutate as _,
        },
    },
};

type AssetIdOf<T, I> = <T as pallet_assets::Config<I>>::AssetId;
type AssetBalanceOf<T, I> = <T as pallet_assets::Config<I>>::Balance;

/// Assets API for the runtime.
///
/// Provides methods to create, mint, and manage assets in `pallet-assets`.
pub trait AssetsAPI<T, I = pallet_assets::Instance1>
where
    T: RuntimeEnv,
    T::Runtime: pallet_assets::Config<I>,
    I: 'static,
{
    /// Creates `value` amount of tokens and assigns them to `account`, increasing the
    /// total supply.
    ///
    /// # Arguments
    /// * `id` - ID of the new asset to be created.
    /// * `owner` - The owner of the created asset (accepts any type convertible to
    ///   AccountId).
    /// * `min_balance` - The asset amount one account need at least.
    fn create(
        &mut self,
        id: &AssetIdOf<T::Runtime, I>,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        min_balance: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError>;

    /// Sets the metadata for an existing fungible asset.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `owner` - The owner of the asset (accepts any type convertible to AccountId).
    /// * `name` - Token name.
    /// * `symbol` - Token symbol.
    /// * `decimals` - Token decimals.
    fn set_metadata(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
    ) -> Result<(), DispatchError>;

    /// Returns the metadata for an asset.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    ///
    /// # Returns
    /// A tuple of (name, symbol, decimals).
    fn metadata(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> (Vec<u8>, Vec<u8>, u8);

    /// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
    ///
    /// Successive calls of this method overwrite previous values.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `owner` - The account that owns the tokens (accepts any type convertible to
    ///   AccountId).
    /// * `delegate` - The account that is allowed to spend the tokens (accepts any type
    ///   convertible to AccountId).
    /// * `amount` - The number of tokens to approve.
    fn approve(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        delegate: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        amount: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError>;

    /// Creates `value` amount of tokens and assigns them to `account`, increasing the
    /// total supply.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `account` - The account to be credited with the created tokens (accepts any type
    ///   convertible to AccountId).
    /// * `value` - The number of tokens to mint.
    fn mint_into(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        account: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        value: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<AssetBalanceOf<T::Runtime, I>, DispatchError>;

    /// Transfer `amount` of tokens from `origin` to `dest`.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `source` - The account from which tokens are transferred (accepts any type
    ///   convertible to AccountId).
    /// * `dest` - The account to which tokens are transferred (accepts any type
    ///   convertible to AccountId).
    /// * `amount` - The number of tokens to transfer.
    fn transfer(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        source: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        dest: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        amount: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError>;

    /// Returns the account balance for the specified `owner`.
    ///
    /// # Arguments
    /// * `owner` - The account whose balance is being queried (accepts any type
    ///   convertible to AccountId).
    fn balance_of(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
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
    /// * `owner` - The account that owns the tokens (accepts any type convertible to
    ///   AccountId).
    /// * `delegate` - The account that is allowed to spend the tokens (accepts any type
    ///   convertible to AccountId).
    fn allowance(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        delegate: impl IntoAccountId<AccountIdFor<T::Runtime>>,
    ) -> AssetBalanceOf<T::Runtime, I>;

    /// Check if the asset exists.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    fn asset_exists(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> bool;
}

impl<T, I> AssetsAPI<T, I> for T
where
    T: RuntimeEnv,
    T::Runtime: pallet_assets::Config<I>,
    I: 'static,
{
    fn create(
        &mut self,
        id: &AssetIdOf<T::Runtime, I>,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        min_balance: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError> {
        let owner = owner.into_account_id();
        self.execute_with(|| {
            <pallet_assets::Pallet<T::Runtime, I> as Create<
                AccountIdFor<T::Runtime>,
            >>::create(id.clone(), owner, true, min_balance)
        })
    }

    fn set_metadata(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
    ) -> Result<(), DispatchError> {
        let owner = owner.into_account_id();
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::set(
                asset.clone().into(),
                &owner,
                name,
                symbol,
                decimals,
            )
        })
    }

    fn metadata(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> (Vec<u8>, Vec<u8>, u8) {
        self.execute_with(|| {
            let name = <pallet_assets::Pallet<T::Runtime, I> as MetadataInspect<
                AccountIdFor<T::Runtime>,
            >>::name(asset.clone());
            let symbol = <pallet_assets::Pallet<T::Runtime, I> as MetadataInspect<
                AccountIdFor<T::Runtime>,
            >>::symbol(asset.clone());
            let decimals = <pallet_assets::Pallet<T::Runtime, I> as MetadataInspect<
                AccountIdFor<T::Runtime>,
            >>::decimals(asset.clone());
            (name, symbol, decimals)
        })
    }

    fn mint_into(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        account: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        value: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<AssetBalanceOf<T::Runtime, I>, DispatchError> {
        let account = account.into_account_id();
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::mint_into(
                asset.clone(),
                &account,
                value,
            )
        })
    }

    fn transfer(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        source: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        dest: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        amount: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError> {
        let source = source.into_account_id();
        let dest = dest.into_account_id();
        self.execute_with(|| {
            <pallet_assets::Pallet<T::Runtime, I> as Mutate<AccountIdFor<T::Runtime>>>::transfer(
                asset.clone(),
                &source,
                &dest,
                amount,
                frame_support::traits::tokens::Preservation::Preserve,
            ).map(|_| ())
        })
    }

    fn approve(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        delegate: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        amount: AssetBalanceOf<T::Runtime, I>,
    ) -> Result<(), DispatchError> {
        let owner = owner.into_account_id();
        let delegate = delegate.into_account_id();
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::approve(
                asset.clone(),
                &owner,
                &delegate,
                amount,
            )
        })
    }

    fn balance_of(
        &mut self,
        asset: &AssetIdOf<T::Runtime, I>,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
    ) -> AssetBalanceOf<T::Runtime, I> {
        let owner = owner.into_account_id();
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::balance(asset.clone(), &owner)
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
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        delegate: impl IntoAccountId<AccountIdFor<T::Runtime>>,
    ) -> AssetBalanceOf<T::Runtime, I> {
        let owner = owner.into_account_id();
        let delegate = delegate.into_account_id();
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, I>::allowance(
                asset.clone(),
                &owner,
                &delegate,
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
    use crate::{
        DefaultRuntime,
        DefaultRuntime::default_actor,
    };

    #[test]
    fn create_works() {
        let mut runtime = DefaultRuntime::default();
        let admin = default_actor();
        let asset_id = 1u32;
        let min_balance = 1u128;

        let result = runtime.create(&asset_id, &admin, min_balance);

        assert!(result.is_ok());
        assert!(runtime.asset_exists(&asset_id));
    }

    #[test]
    fn set_metadata_works() {
        let mut runtime = DefaultRuntime::default();
        let admin = default_actor();
        let asset_id = 1u32;

        runtime.create(&asset_id, &admin, 1u128).unwrap();

        let name = b"Test Token".to_vec();
        let symbol = b"TEST".to_vec();
        let decimals = 18u8;

        let result = runtime.set_metadata(&asset_id, &admin, name, symbol, decimals);

        assert!(result.is_ok());
    }

    #[test]
    fn metadata_works() {
        let mut runtime = DefaultRuntime::default();
        let admin = default_actor();
        let asset_id = 1u32;

        runtime.create(&asset_id, &admin, 1u128).unwrap();

        let name = b"Test Token".to_vec();
        let symbol = b"TEST".to_vec();
        let decimals = 18u8;

        runtime
            .set_metadata(&asset_id, &admin, name.clone(), symbol.clone(), decimals)
            .unwrap();

        let (retrieved_name, retrieved_symbol, retrieved_decimals) =
            runtime.metadata(&asset_id);

        assert_eq!(retrieved_name, name);
        assert_eq!(retrieved_symbol, symbol);
        assert_eq!(retrieved_decimals, decimals);
    }

    #[test]
    fn approve_works() {
        let mut runtime = DefaultRuntime::default();
        let admin = default_actor();
        let spender = ink_e2e::bob().into_account_id();
        let asset_id = 1u32;

        runtime.create(&asset_id, &admin, 1u128).unwrap();
        runtime.mint_into(&asset_id, &admin, 1000u128).unwrap();

        let allowance_before = runtime.allowance(&asset_id, &admin, &spender);
        assert_eq!(allowance_before, 0);

        let result = runtime.approve(&asset_id, &admin, &spender, 500u128);

        assert!(result.is_ok());

        let allowance_after = runtime.allowance(&asset_id, &admin, &spender);
        assert_eq!(allowance_after, 500);
    }

    #[test]
    fn mint_into_works() {
        let mut runtime = DefaultRuntime::default();
        let admin = default_actor();
        let asset_id = 1u32;

        runtime.create(&asset_id, &admin, 1u128).unwrap();

        let balance_before = runtime.balance_of(&asset_id, &admin);
        assert_eq!(balance_before, 0);

        runtime.mint_into(&asset_id, &admin, 100u128).unwrap();

        let balance_after = runtime.balance_of(&asset_id, &admin);
        assert_eq!(balance_after, 100);
    }

    #[test]
    fn transfer_works() {
        let mut runtime = DefaultRuntime::default();
        let admin = default_actor();
        let recipient = ink_e2e::bob().into_account_id();
        let asset_id = 1u32;

        runtime.create(&asset_id, &admin, 1u128).unwrap();
        runtime.mint_into(&asset_id, &admin, 1000u128).unwrap();

        let admin_balance_before = runtime.balance_of(&asset_id, &admin);
        let recipient_balance_before = runtime.balance_of(&asset_id, &recipient);

        assert_eq!(admin_balance_before, 1000);
        assert_eq!(recipient_balance_before, 0);

        let result = runtime.transfer(&asset_id, &admin, &recipient, 300u128);

        assert!(result.is_ok());

        let admin_balance_after = runtime.balance_of(&asset_id, &admin);
        let recipient_balance_after = runtime.balance_of(&asset_id, &recipient);

        assert_eq!(admin_balance_after, 700);
        assert_eq!(recipient_balance_after, 300);
    }

    #[test]
    fn balance_of_works() {
        let mut runtime = DefaultRuntime::default();
        let admin = default_actor();
        let asset_id = 1u32;

        runtime.create(&asset_id, &admin, 1u128).unwrap();

        let balance = runtime.balance_of(&asset_id, &admin);
        assert_eq!(balance, 0);

        runtime.mint_into(&asset_id, &admin, 500u128).unwrap();

        let balance = runtime.balance_of(&asset_id, &admin);
        assert_eq!(balance, 500);
    }

    #[test]
    fn total_supply_works() {
        let mut runtime = DefaultRuntime::default();
        let admin = default_actor();
        let asset_id = 1u32;

        runtime.create(&asset_id, &admin, 1u128).unwrap();

        let supply_before = runtime.total_supply(&asset_id);
        assert_eq!(supply_before, 0);

        runtime.mint_into(&asset_id, &admin, 1000u128).unwrap();

        let supply_after = runtime.total_supply(&asset_id);
        assert_eq!(supply_after, 1000);
    }

    #[test]
    fn allowance_works() {
        let mut runtime = DefaultRuntime::default();
        let admin = default_actor();
        let spender = ink_e2e::bob().into_account_id();
        let asset_id = 1u32;

        runtime.create(&asset_id, &admin, 1u128).unwrap();

        let allowance = runtime.allowance(&asset_id, &admin, &spender);
        assert_eq!(allowance, 0);

        runtime
            .approve(&asset_id, &admin, &spender, 250u128)
            .unwrap();

        let allowance = runtime.allowance(&asset_id, &admin, &spender);
        assert_eq!(allowance, 250);
    }

    #[test]
    fn asset_exists_works() {
        let mut runtime = DefaultRuntime::default();
        let admin = default_actor();
        let asset_id = 1u32;

        assert!(!runtime.asset_exists(&asset_id));

        runtime.create(&asset_id, &admin, 1u128).unwrap();

        assert!(runtime.asset_exists(&asset_id));
    }
}
