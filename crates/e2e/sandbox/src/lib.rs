use core::any::Any;

pub mod api;
pub mod macros;
pub mod prelude {
    pub use crate::api::{
        balance_api::BalanceAPI,
        contracts_api::ContractAPI,
        system_api::SystemAPI,
        timestamp_api::TimestampAPI,
    };
}

pub use macros::{
    BlockBuilder,
    MinimalSandbox,
};

/// Export pallets that are used in [`macros::create_minimal_sandbox`]
pub use {
    frame_support,
    frame_system,
    pallet_balances,
    pallet_contracts,
    pallet_timestamp,
    paste,
    sp_externalities::Extension,
    sp_io::TestExternalities,
};

pub use frame_metadata::RuntimeMetadataPrefixed;
pub use frame_support::weights::Weight;

use frame_support::{
    sp_runtime::traits::Dispatchable,
    traits::fungible::Inspect,
};
use frame_system::{
    pallet_prelude::BlockNumberFor,
    EventRecord,
};
use pallet_contracts::{
    ContractExecResult,
    ContractInstantiateResult,
};

type BalanceOf<R> =
    <<R as pallet_contracts::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;

/// The type of an account identifier.
pub type AccountIdFor<R> = <R as frame_system::Config>::AccountId;

/// Alias for `frame-system`'s `RuntimeCall` type.
pub type RuntimeCall<R> = <R as frame_system::Config>::RuntimeCall;

pub type EventRecordOf<Runtime> = EventRecord<
    <Runtime as frame_system::Config>::RuntimeEvent,
    <Runtime as frame_system::Config>::Hash,
>;

pub type ContractInstantiateResultFor<Runtime> = ContractInstantiateResult<
    AccountIdFor<Runtime>,
    BalanceOf<Runtime>,
    EventRecordOf<Runtime>,
>;

pub type ContractExecResultFor<Runtime> =
    ContractExecResult<BalanceOf<Runtime>, EventRecordOf<Runtime>>;

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
        Weight::from_parts(100_000_000_000, 3 * 1024 * 1024)
    }

    /// Metadata of the runtime.
    fn get_metadata() -> RuntimeMetadataPrefixed;

    /// Convert an account to an call origin.
    fn convert_account_to_origin(
        account: AccountIdFor<Self::Runtime>,
    ) -> <<Self::Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin;
}
