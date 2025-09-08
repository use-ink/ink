use core::any::Any;

pub mod api;
pub mod macros;

pub use frame_metadata::RuntimeMetadataPrefixed;
pub use frame_support::weights::Weight;
use frame_support::{
    sp_runtime::traits::Dispatchable,
    traits::fungible::Inspect,
};
use frame_system::{
    EventRecord,
    pallet_prelude::{
        BlockNumberFor,
        OriginFor,
    },
};
use ink_primitives::U256;
pub use macros::{
    BlockBuilder,
    DefaultSandbox,
};
use pallet_revive::{
    ContractResult,
    ExecReturnValue,
    InstantiateReturnValue,
};
use sp_core::Get;
/// Export pallets that are used in [`crate::create_sandbox`]
pub use {
    frame_support::sp_runtime::testing::H256,
    frame_support::{
        self,
        sp_runtime::{
            AccountId32,
            DispatchError,
        },
    },
    frame_system,
    pallet_balances,
    pallet_revive,
    pallet_timestamp,
    paste,
    sp_core::crypto::Ss58Codec,
    sp_externalities::{
        self,
        Extension,
    },
    sp_io::TestExternalities,
};

/// A snapshot of the storage.
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// The storage raw key-value pairs.
    pub storage: RawStorage,
    /// The storage root hash.
    pub storage_root: StorageRoot,
}

pub type RawStorage = Vec<(Vec<u8>, (Vec<u8>, i32))>;
pub type StorageRoot = H256;

/// Alias for the balance type.
type BalanceOf<R> =
    <<R as pallet_revive::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;

/// Alias for the account ID type.
pub type AccountIdFor<R> = <R as frame_system::Config>::AccountId;

/// Alias for the runtime call type.
pub type RuntimeCall<R> = <R as frame_system::Config>::RuntimeCall;

/// Alias for the event record type.
pub type EventRecordOf<Runtime> = EventRecord<
    <Runtime as frame_system::Config>::RuntimeEvent,
    <Runtime as frame_system::Config>::Hash,
>;

/// Alias for the contract instantiate result.
pub type ContractInstantiateResultFor<Runtime> =
    ContractResult<OriginFor<Runtime>, BalanceOf<Runtime>>;

pub type ContractResultFor<Runtime> = ContractResult<Runtime, BalanceOf<Runtime>>;

pub type ContractResultInstantiate<Runtime> =
    ContractResult<InstantiateReturnValue, BalanceOf<Runtime>>;

/// Alias for the contract exec result.
pub type ContractExecResultFor<Runtime> =
    ContractResult<ExecReturnValue, BalanceOf<Runtime>>;

/// Alias for the `map_account` result.
pub type MapAccountResultFor = Result<(), DispatchError>;

/// Alias for the runtime of a sandbox.
pub type RuntimeOf<S> = <S as Sandbox>::Runtime;

/// Alias for the runtime event of a sandbox.
pub type RuntimeEventOf<S> = <RuntimeOf<S> as frame_system::Config>::RuntimeEvent;

/// Sandbox defines the API of a sandboxed runtime.
pub trait Sandbox {
    /// The runtime associated with the sandbox.
    type Runtime: frame_system::Config;

    /// Execute the given externalities.
    fn execute_with<T>(&mut self, execute: impl FnOnce() -> T) -> T;

    /// Dry run an action without modifying the storage.
    fn dry_run<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T;

    /// Register an extension.
    fn register_extension<E: Any + Extension>(&mut self, ext: E);

    /// Initialize a new block at particular height.
    fn initialize_block(
        _height: BlockNumberFor<Self::Runtime>,
        _parent_hash: <Self::Runtime as frame_system::Config>::Hash,
    ) {
    }

    /// Finalize a block at particular height.
    fn finalize_block(
        _height: BlockNumberFor<Self::Runtime>,
    ) -> <Self::Runtime as frame_system::Config>::Hash {
        Default::default()
    }

    /// Default actor for the sandbox.
    fn default_actor() -> AccountIdFor<Self::Runtime>;

    fn default_gas_limit() -> Weight {
        Weight::from_parts(100_000_000_000_000, 6 * 1024 * 1024)
    }

    /// Metadata of the runtime.
    fn get_metadata() -> RuntimeMetadataPrefixed;

    /// Convert an account to a call origin.
    fn convert_account_to_origin(
        account: AccountIdFor<Self::Runtime>,
    ) -> <<Self::Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin;

    /// Take a snapshot of the storage.
    fn take_snapshot(&mut self) -> Snapshot;

    /// Restore the storage from the given snapshot.
    fn restore_snapshot(&mut self, snapshot: Snapshot);
}

/// Converts from the generic `Balance` type to the Ethereum native `U256`.
///
/// # Developer Note
///
/// `pallet-revive` uses both types, hence we have to convert in between them
/// for certain functions. Notice that precision loss might occur when converting
/// the other way (from `U256` to `Balance`).
///
/// See <https://github.com/paritytech/polkadot-sdk/pull/9101> for more details.
pub fn balance_to_evm_value<R>(value: BalanceOf<R>) -> U256
where
    R: pallet_revive::Config,
    BalanceOf<R>: Into<U256>,
    U256: From<u32>,
{
    let native_to_eth_ratio: U256 =
        <R as pallet_revive::Config>::NativeToEthRatio::get().into();
    let evm_value: U256 = value.into();
    native_to_eth_ratio.saturating_mul(evm_value)
}
