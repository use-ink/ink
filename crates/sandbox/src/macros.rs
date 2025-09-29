use std::time::SystemTime;

use frame_support::{
    sp_runtime::{
        BuildStorage,
        traits::{
            Header,
            One,
        },
    },
    traits::Hooks,
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_io::TestExternalities;

/// A helper struct for initializing and finalizing blocks.
pub struct BlockBuilder<T>(std::marker::PhantomData<T>);

impl<
    T: pallet_balances::Config
        + pallet_timestamp::Config<Moment = u64>
        + pallet_revive::Config,
> BlockBuilder<T>
{
    /// Create a new externalities with the given balances.
    pub fn new_ext(balances: Vec<(T::AccountId, T::Balance)>) -> TestExternalities {
        let mut storage = frame_system::GenesisConfig::<T>::default()
            .build_storage()
            .unwrap();

        pallet_balances::GenesisConfig::<T> {
            balances,
            dev_accounts: None,
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        let mut ext = TestExternalities::new(storage);

        ext.execute_with(|| {
            Self::initialize_block(BlockNumberFor::<T>::one(), Default::default())
        });
        ext
    }

    /// Initialize a new block at particular height.
    pub fn initialize_block(
        height: frame_system::pallet_prelude::BlockNumberFor<T>,
        parent_hash: <T as frame_system::Config>::Hash,
    ) {
        frame_system::Pallet::<T>::reset_events();
        frame_system::Pallet::<T>::initialize(&height, &parent_hash, &Default::default());
        pallet_balances::Pallet::<T>::on_initialize(height);
        pallet_timestamp::Pallet::<T>::set_timestamp(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        );
        pallet_timestamp::Pallet::<T>::on_initialize(height);
        pallet_revive::Pallet::<T>::on_initialize(height);
        frame_system::Pallet::<T>::note_finished_initialize();
    }

    /// Finalize a block at particular height.
    pub fn finalize_block(
        height: frame_system::pallet_prelude::BlockNumberFor<T>,
    ) -> <T as frame_system::Config>::Hash {
        pallet_revive::Pallet::<T>::on_finalize(height);
        use sp_core::Get;
        let minimum_period = <T as pallet_timestamp::Config>::MinimumPeriod::get();
        let now = pallet_timestamp::Pallet::<T>::get()
            .checked_add(minimum_period)
            .unwrap();
        pallet_timestamp::Pallet::<T>::set_timestamp(now);
        pallet_timestamp::Pallet::<T>::on_finalize(height);
        pallet_balances::Pallet::<T>::on_finalize(height);
        frame_system::Pallet::<T>::finalize().hash()
    }
}

/// Macro creating a minimal runtime with the given name.
///
/// The new macro will automatically implement `crate::Sandbox`.
#[macro_export]
macro_rules! create_sandbox {
    ($name:ident) => {
        $crate::paste::paste! {
            $crate::create_sandbox!($name, [<$name Runtime>], (), {});
        }
    };
    ($name:ident, $debug: ty) => {
        $crate::paste::paste! {
            $crate::create_sandbox!($name, [<$name Runtime>], $debug, {});
        }
    };
    ($name:ident, $debug: ty, { $( $pallet_name:tt : $pallet:ident ),* $(,)? }) => {
        $crate::paste::paste! {
            $crate::create_sandbox!($name, [<$name Runtime>], $debug, {
                $(
                    $pallet_name : $pallet,
                )*
            });
        }
    };
    ($sandbox:ident, $runtime:ident, $debug: ty, { $( $pallet_name:tt : $pallet:ident ),* $(,)? }) => {

// Put all the boilerplate into an auxiliary module
mod construct_runtime {

    // Bring some common types into the scope
    use $crate::frame_support::{
        construct_runtime,
        derive_impl,
        parameter_types,
        sp_runtime::{
            traits::Convert,
            AccountId32, Perbill,
        },
        traits::{ConstBool, ConstU128, ConstU32, ConstU64, Currency},
        weights::Weight,
    };

    use $crate::Snapshot;

    // Define the runtime type as a collection of pallets
    construct_runtime!(
        pub enum $runtime {
            System: $crate::frame_system,
            Balances: $crate::pallet_balances,
            Timestamp: $crate::pallet_timestamp,
            Revive: $crate::pallet_revive,
            $(
                $pallet_name: $pallet,
            )*
        }
    );

    // Configure pallet system
    #[derive_impl($crate::frame_system::config_preludes::SolochainDefaultConfig as $crate::frame_system::DefaultConfig)]
    impl $crate::frame_system::Config for $runtime {
        type Block = $crate::frame_system::mocking::MockBlockU32<$runtime>;
        type Version = ();
        type BlockHashCount = ConstU32<250>;
        type AccountData = $crate::pallet_balances::AccountData<<$runtime as $crate::pallet_balances::Config>::Balance>;
    }

    // Configure pallet balances
    impl $crate::pallet_balances::Config for $runtime {
        type RuntimeEvent = RuntimeEvent;
        type WeightInfo = ();
        type Balance = u128;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = ();
        type MaxLocks = ();
        type MaxReserves = ();
        type MaxFreezes = ();
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
    }

    // Configure pallet timestamp
    impl $crate::pallet_timestamp::Config for $runtime {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = ConstU64<1>;
        type WeightInfo = ();
    }

    // Configure pallet revive
    type BalanceOf = <Balances as Currency<AccountId32>>::Balance;
    impl Convert<Weight, BalanceOf> for $runtime {
        fn convert(w: Weight) -> BalanceOf {
            w.ref_time().into()
        }
    }

    parameter_types! {
        pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
    }

    impl $crate::pallet_revive::Config for $runtime {
        type AddressMapper = $crate::pallet_revive::AccountId32Mapper<Self>;
        type ChainId = ConstU64<1>;
        type NativeToEthRatio = ConstU32<100_000_000>;
        type Time = Timestamp;
        type Currency = Balances;
        type RuntimeEvent = RuntimeEvent;
        type RuntimeCall = RuntimeCall;
        type DepositPerItem = ConstU128<1>;
        type DepositPerByte = ConstU128<1>;
        type WeightPrice = Self;
        type WeightInfo = ();
        type RuntimeMemory = ConstU32<{ 128 * 1024 * 1024 }>;
        type PVFMemory = ConstU32<{ 512 * 1024 * 1024 }>;
        type UnsafeUnstableInterface = ConstBool<true>;
        type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
        type RuntimeHoldReason = RuntimeHoldReason;
        type UploadOrigin = $crate::frame_system::EnsureSigned<Self::AccountId>;
        type InstantiateOrigin = $crate::frame_system::EnsureSigned<Self::AccountId>;
        type EthGasEncoder = ();
        type FindAuthor = ();
        type Precompiles = (
            // todo
            //ERC20<Self, InlineIdConfig<0x120>, TrustBackedAssetsInstance>,
            //ERC20<Self, InlineIdConfig<0x320>, PoolAssetsInstance>,
            //XcmPrecompile<Self>,
        );
        type AllowEVMBytecode = ConstBool<false>;
    }

    /// Default initial balance for the default account.
    pub const INITIAL_BALANCE: u128 = 1_000_000_000_000_000;
    pub const DEFAULT_ACCOUNT: AccountId32 = AccountId32::new([1u8; 32]);

    pub struct $sandbox {
        ext: $crate::TestExternalities,
    }

    impl ::std::default::Default for $sandbox {
        fn default() -> Self {
            let ext = $crate::macros::BlockBuilder::<$runtime>::new_ext(vec![(
                DEFAULT_ACCOUNT,
                INITIAL_BALANCE,
            )]);
            Self { ext }
        }
    }

    // Implement `crate::Sandbox` trait
    impl $crate::Sandbox for $sandbox {
        type Runtime = $runtime;

        fn execute_with<T>(&mut self, execute: impl FnOnce() -> T) -> T {
            self.ext.execute_with(execute)
        }

        fn dry_run<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T {
            // Make a backup of the backend.
            let backend_backup = self.ext.as_backend();
            // Run the action, potentially modifying storage. Ensure, that there are no pending changes
            // that would affect the reverted backend.
            let result = action(self);
            self.ext.commit_all().expect("Failed to commit changes");

            // Restore the backend.
            self.ext.backend = backend_backup;
            result
        }

        fn register_extension<E: ::core::any::Any + $crate::Extension>(&mut self, ext: E) {
            self.ext.register_extension(ext);
        }

        fn initialize_block(
            height: $crate::frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
            parent_hash: <Self::Runtime as $crate::frame_system::Config>::Hash,
        ) {
            $crate::macros::BlockBuilder::<Self::Runtime>::initialize_block(height, parent_hash)
        }

        fn finalize_block(
            height: $crate::frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
        ) -> <Self::Runtime as $crate::frame_system::Config>::Hash {
            $crate::macros::BlockBuilder::<Self::Runtime>::finalize_block(height)
        }

        fn default_actor() -> $crate::AccountIdFor<Self::Runtime> {
            DEFAULT_ACCOUNT
        }

        fn get_metadata() -> $crate::RuntimeMetadataPrefixed {
            Self::Runtime::metadata()
        }

        fn convert_account_to_origin(
            account: $crate::AccountIdFor<Self::Runtime>,
        ) -> <<Self::Runtime as $crate::frame_system::Config>::RuntimeCall as $crate::frame_support::sp_runtime::traits::Dispatchable>::RuntimeOrigin {
            Some(account).into()
        }

        fn take_snapshot(&mut self) -> Snapshot {
            let mut backend = self.ext.as_backend().clone();
            let raw_key_values = backend
                .backend_storage_mut()
                .drain()
                .into_iter()
                .filter(|(_, (_, r))| *r > 0)
                .collect::<Vec<(Vec<u8>, (Vec<u8>, i32))>>();
            let root = backend.root().to_owned();
            Snapshot {
                storage: raw_key_values,
                storage_root: root,
            }
        }

        fn restore_snapshot(&mut self, snapshot: Snapshot) {
            self.ext = $crate::TestExternalities::from_raw_snapshot(
                snapshot.storage,
                snapshot.storage_root,
                Default::default(),
            );
        }
    }
}

// Export runtime type itself, pallets and useful types from the auxiliary module
pub use construct_runtime::{
    $sandbox, $runtime, Balances, Revive, PalletInfo, RuntimeCall, RuntimeEvent, RuntimeHoldReason,
    RuntimeOrigin, System, Timestamp,
};
    };
}

create_sandbox!(DefaultSandbox);
