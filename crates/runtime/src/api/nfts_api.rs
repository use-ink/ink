use crate::{
    AccountIdFor,
    IntoAccountId,
    RuntimeEnv,
};
use frame_support::{
    pallet_prelude::DispatchError,
    traits::tokens::nonfungibles_v2::{
        Create,
        Inspect,
        Mutate,
        Transfer,
    },
};
use pallet_nfts::{
    CollectionConfigFor,
    CollectionDetailsFor,
    ItemConfig,
    ItemSettings,
};

type CollectionIdOf<T, I> = <T as pallet_nfts::Config<I>>::CollectionId;
type ItemIdOf<T, I> = <T as pallet_nfts::Config<I>>::ItemId;

/// NFTs API for the runtime.
///
/// Provides helpers to create collections and manipulate items in `pallet-nfts`
/// when running against the in-memory runtime backend.
pub trait NftsAPI<T, I = ()>
where
    T: RuntimeEnv,
    T::Runtime: pallet_nfts::Config<I>,
    I: 'static,
{
    /// Creates a new collection owned by `owner` and administered by `admin`.
    ///
    /// This uses the pallet's `Create` implementation directly, so the caller
    /// must provide a config where `CollectionSetting::DepositRequired` is not
    /// disabled.
    fn create_collection(
        &mut self,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        admin: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        config: CollectionConfigFor<T::Runtime, I>,
    ) -> Result<CollectionIdOf<T::Runtime, I>, DispatchError>;

    /// Mints an item into `beneficiary`.
    ///
    /// `deposit_to_collection_owner` controls whether the deposit is taken from
    /// the collection owner instead of the beneficiary.
    fn mint_into(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
        item: ItemIdOf<T::Runtime, I>,
        beneficiary: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        deposit_to_collection_owner: bool,
    ) -> Result<(), DispatchError>;

    /// Transfers ownership of an item.
    fn transfer(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
        item: ItemIdOf<T::Runtime, I>,
        destination: impl IntoAccountId<AccountIdFor<T::Runtime>>,
    ) -> Result<(), DispatchError>;

    /// Burns an item, optionally enforcing ownership.
    fn burn(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
        item: ItemIdOf<T::Runtime, I>,
        maybe_check_owner: Option<impl IntoAccountId<AccountIdFor<T::Runtime>>>,
    ) -> Result<(), DispatchError>;

    /// Returns the owner of an item, if any.
    fn owner_of(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
        item: &ItemIdOf<T::Runtime, I>,
    ) -> Option<AccountIdFor<T::Runtime>>;

    /// Returns collection details, if the collection exists.
    fn collection_details(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
    ) -> Option<CollectionDetailsFor<T::Runtime, I>>;

    /// Returns the config for a collection, if it exists.
    fn collection_config(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
    ) -> Option<CollectionConfigFor<T::Runtime, I>>;

    /// Returns the next collection ID.
    fn next_collection_id(&mut self) -> Option<CollectionIdOf<T::Runtime, I>>;

    /// Checks if a collection exists.
    fn collection_exists(&mut self, collection: &CollectionIdOf<T::Runtime, I>) -> bool;

    /// Checks if an item exists inside a collection.
    fn item_exists(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
        item: &ItemIdOf<T::Runtime, I>,
    ) -> bool;
}

impl<T, I> NftsAPI<T, I> for T
where
    T: RuntimeEnv,
    T::Runtime: pallet_nfts::Config<I>,
    I: 'static,
{
    fn create_collection(
        &mut self,
        owner: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        admin: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        config: CollectionConfigFor<T::Runtime, I>,
    ) -> Result<CollectionIdOf<T::Runtime, I>, DispatchError> {
        let owner = owner.into_account_id();
        let admin = admin.into_account_id();
        self.execute_with(|| {
            <pallet_nfts::Pallet<T::Runtime, I> as Create<
                AccountIdFor<T::Runtime>,
                CollectionConfigFor<T::Runtime, I>,
            >>::create_collection(&owner, &admin, &config)
        })
    }

    fn mint_into(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
        item: ItemIdOf<T::Runtime, I>,
        beneficiary: impl IntoAccountId<AccountIdFor<T::Runtime>>,
        deposit_to_collection_owner: bool,
    ) -> Result<(), DispatchError> {
        let beneficiary = beneficiary.into_account_id();
        let item_config = ItemConfig {
            settings: ItemSettings::all_enabled(),
        };
        self.execute_with(|| {
            <pallet_nfts::Pallet<T::Runtime, I> as Mutate<
                AccountIdFor<T::Runtime>,
                ItemConfig,
            >>::mint_into(
                collection,
                &item,
                &beneficiary,
                &item_config,
                deposit_to_collection_owner,
            )
        })
    }

    fn transfer(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
        item: ItemIdOf<T::Runtime, I>,
        destination: impl IntoAccountId<AccountIdFor<T::Runtime>>,
    ) -> Result<(), DispatchError> {
        let destination = destination.into_account_id();
        self.execute_with(|| {
            <pallet_nfts::Pallet<T::Runtime, I> as Transfer<
                AccountIdFor<T::Runtime>,
            >>::transfer(collection, &item, &destination)
        })
    }

    fn burn(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
        item: ItemIdOf<T::Runtime, I>,
        maybe_check_owner: Option<impl IntoAccountId<AccountIdFor<T::Runtime>>>,
    ) -> Result<(), DispatchError> {
        let maybe_owner = maybe_check_owner.map(|owner| owner.into_account_id());
        self.execute_with(|| {
            <pallet_nfts::Pallet<T::Runtime, I> as Mutate<
                AccountIdFor<T::Runtime>,
                ItemConfig,
            >>::burn(collection, &item, maybe_owner.as_ref())
        })
    }

    fn owner_of(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
        item: &ItemIdOf<T::Runtime, I>,
    ) -> Option<AccountIdFor<T::Runtime>> {
        self.execute_with(|| {
            <pallet_nfts::Pallet<T::Runtime, I> as Inspect<AccountIdFor<T::Runtime>>>::owner(
                collection, item,
            )
        })
    }

    fn collection_details(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
    ) -> Option<CollectionDetailsFor<T::Runtime, I>> {
        self.execute_with(|| pallet_nfts::Collection::<T::Runtime, I>::get(collection))
    }

    fn collection_config(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
    ) -> Option<CollectionConfigFor<T::Runtime, I>> {
        self.execute_with(|| {
            pallet_nfts::CollectionConfigOf::<T::Runtime, I>::get(collection)
        })
    }

    fn next_collection_id(&mut self) -> Option<CollectionIdOf<T::Runtime, I>> {
        self.execute_with(|| pallet_nfts::NextCollectionId::<T::Runtime, I>::get())
    }

    fn collection_exists(&mut self, collection: &CollectionIdOf<T::Runtime, I>) -> bool {
        self.execute_with(|| {
            pallet_nfts::Collection::<T::Runtime, I>::contains_key(collection)
        })
    }

    fn item_exists(
        &mut self,
        collection: &CollectionIdOf<T::Runtime, I>,
        item: &ItemIdOf<T::Runtime, I>,
    ) -> bool {
        self.execute_with(|| {
            pallet_nfts::Item::<T::Runtime, I>::contains_key(collection, item)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DefaultRuntime;
    use pallet_nfts::{
        CollectionConfig,
        CollectionSettings,
        MintSettings,
    };

    type Runtime = <DefaultRuntime as RuntimeEnv>::Runtime;

    fn simple_config() -> CollectionConfigFor<Runtime> {
        CollectionConfig {
            settings: CollectionSettings::all_enabled(),
            max_supply: None,
            mint_settings: MintSettings::default(),
        }
    }

    #[test]
    fn create_and_mint_work() {
        let mut runtime = DefaultRuntime::default();
        let owner = DefaultRuntime::default_actor();
        let collection_id = runtime
            .create_collection(&owner, &owner, simple_config())
            .expect("create failed");

        assert_eq!(collection_id, 0);
        assert!(runtime.collection_exists(&collection_id));

        runtime
            .mint_into(&collection_id, 1u32, &owner, false)
            .expect("mint failed");

        assert_eq!(runtime.owner_of(&collection_id, &1u32), Some(owner));
    }

    #[test]
    fn transfer_and_burn_work() {
        let mut runtime = DefaultRuntime::default();
        let owner = DefaultRuntime::default_actor();
        let recipient = ink_e2e::bob().into_account_id();

        let collection = runtime
            .create_collection(&owner, &owner, simple_config())
            .expect("create failed");
        runtime
            .mint_into(&collection, 7u32, &owner, false)
            .expect("mint failed");

        runtime
            .transfer(&collection, 7u32, &recipient)
            .expect("transfer failed");
        assert_eq!(
            runtime.owner_of(&collection, &7u32),
            Some(recipient.clone())
        );

        runtime
            .burn(&collection, 7u32, None::<&AccountIdFor<Runtime>>)
            .expect("burn failed");
        assert!(!runtime.item_exists(&collection, &7u32));
    }
}
