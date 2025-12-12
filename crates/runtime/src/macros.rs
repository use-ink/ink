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

/// Asserts that a contract call succeeded without reverting.
///
/// This macro follows FRAME's `assert_ok!` convention for consistency across
/// the Polkadot ecosystem. It verifies that a contract call completed successfully
/// and did not revert. If the call reverted, the macro panics with a detailed
/// error message extracted from the call trace.
///
/// # Behavior
///
/// - Takes a `CallResult` as input
/// - Checks if `dry_run.did_revert()` is `false`
/// - Panics with error details if the call reverted
/// - Returns the `CallResult` for further inspection if successful
///
/// # Examples
///
/// ```ignore
/// let result = client.call(&alice, &transfer).submit().await?;
/// assert_ok!(result);
/// ```
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {{
        let result = $result;
        if result.dry_run.did_revert() {
            panic!(
                "Expected call to succeed but it reverted.\nError: {:?}",
                result.extract_error()
            );
        }
        result
    }};
    ($result:expr, $($msg:tt)+) => {{
        let result = $result;
        if result.dry_run.did_revert() {
            panic!(
                "{}\nExpected call to succeed but it reverted.\nError: {:?}",
                format_args!($($msg)+),
                result.extract_error()
            );
        }
        result
    }};
}

/// Asserts that a contract call reverted with a specific error.
///
/// This macro follows FRAME's `assert_noop!` convention, which stands for
/// "assert no operation" - meaning the call should fail without changing state.
/// It verifies that a contract call reverted and that the revert reason matches
/// the expected error string.
///
/// # Behavior
///
/// - Takes a `CallResult` and an expected error string as input
/// - Checks if `dry_run.did_revert()` is `true`
/// - Panics if the call succeeded (did not revert)
/// - Extracts the error from the call trace using `extract_error()`
/// - Panics if the actual error doesn't match the expected error
/// - Returns the `CallResult` if both checks pass
///
/// # Examples
///
/// ```ignore
/// let result = client.call(&alice, &insufficient_transfer).submit().await?;
/// assert_noop!(result, "BalanceLow");
/// ```
#[macro_export]
macro_rules! assert_noop {
    ($result:expr, $expected_error:expr) => {{
        let result = $result;
        if !result.dry_run.did_revert() {
            panic!(
                "Expected call to revert with '{}' but it succeeded",
                $expected_error
            );
        }

        let actual_error = result.extract_error();
        if actual_error != Some($expected_error.to_string()) {
            panic!(
                "Expected error '{}' but got {:?}",
                $expected_error,
                actual_error
            );
        }

        result
    }};
    ($result:expr, $expected_error:expr, $($msg:tt)+) => {{
        let result = $result;
        if !result.dry_run.did_revert() {
            panic!(
                "{}\nExpected call to revert with '{}' but it succeeded",
                format_args!($($msg)+),
                $expected_error
            );
        }

        let actual_error = result.extract_error();
        if actual_error != Some($expected_error.to_string()) {
            panic!(
                "{}\nExpected error '{}' but got {:?}",
                format_args!($($msg)+),
                $expected_error,
                actual_error
            );
        }

        result
    }};
}

/// Asserts that a contract call reverted with an error containing a substring.
///
/// Similar to `assert_noop!`, but uses substring matching instead of exact match.
/// This is useful when the exact error format may vary or when checking for partial
/// error information.
///
/// # Behavior
///
/// - Takes a `CallResult` and an expected error substring as input
/// - Checks if `dry_run.did_revert()` is `true`
/// - Panics if the call succeeded (did not revert)
/// - Extracts the error and checks if it contains the expected substring
/// - Returns the `CallResult` if both checks pass
///
/// # Examples
///
/// ```ignore
/// let result = client.call(&alice, &failing_call).submit().await?;
/// assert_noop_contains!(result, "revert");
/// ```
#[macro_export]
macro_rules! assert_noop_contains {
    ($result:expr, $expected_substr:expr) => {{
        let result = $result;
        if !result.dry_run.did_revert() {
            panic!(
                "Expected call to revert with error containing '{}' but it succeeded",
                $expected_substr
            );
        }

        let actual_error = result.extract_error();
        match &actual_error {
            Some(err) if err.contains($expected_substr) => {}
            _ => {
                panic!(
                    "Expected error containing '{}' but got {:?}",
                    $expected_substr, actual_error
                );
            }
        }

        result
    }};
}

/// Asserts that the latest contract event matches an expected event.
///
/// This macro verifies that the last emitted contract event from the runtime
/// matches the provided expected event.
///
/// # Parameters
/// - `client` - Mutable reference to the runtime client
/// - `event` - The expected event
#[macro_export]
macro_rules! assert_last_event {
    ($client:expr, $event:expr $(,)?) => {
        $crate::client::assert_last_contract_event_inner($client, $event)
    };
}

/// A helper struct for initializing and finalizing blocks.
pub struct BlockBuilder<T>(std::marker::PhantomData<T>);

impl<
    T: pallet_balances::Config
        + pallet_timestamp::Config<Moment = u64>
        + pallet_revive::Config,
> BlockBuilder<T>
{
    /// Create a new externalities with the given balances.
    pub fn new_ext(
        balances: Vec<(T::AccountId, <T as pallet_balances::Config>::Balance)>,
    ) -> TestExternalities {
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
/// The new macro will automatically implement `crate::RuntimeEnv`.
#[macro_export]
macro_rules! create_runtime {
    ($name:ident) => {
        $crate::paste::paste! {
            $crate::create_runtime!($name, [<$name Runtime>], (), {});
        }
    };
    ($name:ident, $debug: ty) => {
        $crate::paste::paste! {
            $crate::create_runtime!($name, [<$name Runtime>], $debug, {});
        }
    };
    ($name:ident, $debug: ty, { $( $pallet_name:tt : $pallet:ident ),* $(,)? }) => {
        $crate::paste::paste! {
            $crate::create_runtime!($name, [<$name Runtime>], $debug, {
                $(
                    $pallet_name : $pallet,
                )*
            });
        }
    };
    ($runtime_env:ident, $runtime:ident, $debug: ty, { $( $pallet_name:tt : $pallet:ident ),* $(,)? }) => {

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
            FixedU128,
        },
        traits::{ConstBool, ConstU8, ConstU128, ConstU32, ConstU64, Currency, Everything, Nothing},
        weights::{Weight, IdentityFee},
    };

    use $crate::pallet_transaction_payment::{FungibleAdapter};

    use $crate::Snapshot;

    pub type Balance = u128;

    #[cfg(feature = "xcm")]
    construct_runtime!(
        pub enum $runtime {
            System: $crate::frame_system,
            Balances: $crate::pallet_balances,
            Timestamp: $crate::pallet_timestamp,
            Assets: $crate::pallet_assets::<Instance1>,
            Revive: $crate::pallet_revive,
            TransactionPayment: $crate::pallet_transaction_payment,
            Nfts: $crate::pallet_nfts,
            PolkadotXcm: $crate::pallet_xcm,
            $(
                $pallet_name: $pallet,
            )*
        }
    );

    #[cfg(not(feature = "xcm"))]
    construct_runtime!(
        pub enum $runtime {
            System: $crate::frame_system,
            Balances: $crate::pallet_balances,
            Timestamp: $crate::pallet_timestamp,
            Assets: $crate::pallet_assets::<Instance1>,
            Revive: $crate::pallet_revive,
            TransactionPayment: $crate::pallet_transaction_payment,
            Nfts: $crate::pallet_nfts,
            $(
                $pallet_name: $pallet,
            )*
        }
    );

    #[derive_impl($crate::frame_system::config_preludes::SolochainDefaultConfig as $crate::frame_system::DefaultConfig)]
    impl $crate::frame_system::Config for $runtime {
        type Block = $crate::frame_system::mocking::MockBlockU32<$runtime>;
        type Version = ();
        type BlockHashCount = ConstU32<250>;
        type AccountData = $crate::pallet_balances::AccountData<<$runtime as $crate::pallet_balances::Config>::Balance>;
    }

    impl $crate::pallet_balances::Config for $runtime {
        type RuntimeEvent = RuntimeEvent;
        type WeightInfo = ();
        type Balance = Balance;
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

    impl $crate::pallet_timestamp::Config for $runtime {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = ConstU64<1>;
        type WeightInfo = ();
    }

    // Configure pallet-assets (Instance1 for Trust Backed Assets)
    pub type TrustBackedAssetsInstance = $crate::pallet_assets::Instance1;
    pub type AssetIdForTrustBackedAssets = u32;

    impl $crate::pallet_assets::Config<TrustBackedAssetsInstance> for $runtime {
        type RuntimeEvent = RuntimeEvent;
        type Balance = Balance;
        type AssetId = AssetIdForTrustBackedAssets;
        type AssetIdParameter = $crate::scale::Compact<AssetIdForTrustBackedAssets>;
        type Currency = Balances;
        type CreateOrigin = $crate::frame_support::traits::AsEnsureOriginWithArg<$crate::frame_system::EnsureSigned<AccountId32>>;
        type ForceOrigin = $crate::frame_system::EnsureRoot<AccountId32>;
        type AssetDeposit = ConstU128<1>;
        type AssetAccountDeposit = ConstU128<1>;
        type MetadataDepositBase = ConstU128<1>;
        type MetadataDepositPerByte = ConstU128<1>;
        type ApprovalDeposit = ConstU128<1>;
        type StringLimit = ConstU32<50>;
        type Freezer = ();
        type Holder = ();
        type Extra = ();
        type WeightInfo = ();
        type CallbackHandle = $crate::pallet_assets::AutoIncAssetId<$runtime, TrustBackedAssetsInstance>;
        type RemoveItemsLimit = ConstU32<1000>;
    }

    impl $crate::pallet_transaction_payment::Config for $runtime {
        type RuntimeEvent = RuntimeEvent;
        type OnChargeTransaction = FungibleAdapter<Balances, ()>;
        type OperationalFeeMultiplier = ConstU8<5>;
        type WeightToFee = $crate::pallet_revive::evm::fees::BlockRatioFee<1, 1, Self>;
        type LengthToFee = IdentityFee<Balance>;
        type FeeMultiplierUpdate = ();
        type WeightInfo = $crate::pallet_transaction_payment::weights::SubstrateWeight<$runtime>;
    }

    // Configure pallet-nfts
    pub type NftsCollectionId = u32;
    pub type NftsItemId = u32;

    parameter_types! {
        pub const NftsCollectionDeposit: Balance = 2;
        pub const NftsItemDeposit: Balance = 1;
        pub const NftsMetadataDepositBase: Balance = 1;
        pub const NftsAttributeDepositBase: Balance = 1;
        pub const NftsDepositPerByte: Balance = 1;
        pub const NftsStringLimit: u32 = 50;
        pub const NftsKeyLimit: u32 = 50;
        pub const NftsValueLimit: u32 = 50;
        pub const NftsApprovalsLimit: u32 = 10;
        pub const NftsItemAttributesApprovalsLimit: u32 = 2;
        pub const NftsMaxTips: u32 = 10;
        pub const NftsMaxDeadlineDuration: u32 = 10_000;
        pub const NftsMaxAttributesPerCall: u32 = 2;
        pub NftsFeatures: $crate::pallet_nfts::PalletFeatures = $crate::pallet_nfts::PalletFeatures::all_enabled();
    }

    impl $crate::pallet_nfts::Config for $runtime {
        type RuntimeEvent = RuntimeEvent;
        type CollectionId = NftsCollectionId;
        type ItemId = NftsItemId;
        type Currency = Balances;
        type CreateOrigin = $crate::frame_support::traits::AsEnsureOriginWithArg<$crate::frame_system::EnsureSigned<AccountId32>>;
        type ForceOrigin = $crate::frame_system::EnsureRoot<AccountId32>;
        type Locker = ();
        type CollectionDeposit = NftsCollectionDeposit;
        type ItemDeposit = NftsItemDeposit;
        type MetadataDepositBase = NftsMetadataDepositBase;
        type AttributeDepositBase = NftsAttributeDepositBase;
        type DepositPerByte = NftsDepositPerByte;
        type StringLimit = NftsStringLimit;
        type KeyLimit = NftsKeyLimit;
        type ValueLimit = NftsValueLimit;
        type ApprovalsLimit = NftsApprovalsLimit;
        type ItemAttributesApprovalsLimit = NftsItemAttributesApprovalsLimit;
        type MaxTips = NftsMaxTips;
        type MaxDeadlineDuration = NftsMaxDeadlineDuration;
        type MaxAttributesPerCall = NftsMaxAttributesPerCall;
        type Features = NftsFeatures;
        type OffchainSignature = $crate::frame_support::sp_runtime::MultiSignature;
        type OffchainPublic = <Self::OffchainSignature as $crate::frame_support::sp_runtime::traits::Verify>::Signer;
        #[cfg(feature = "runtime-benchmarks")]
        type Helper = ();
        type WeightInfo = ();
        type BlockNumberProvider = $crate::frame_system::Pallet<$runtime>;
    }

    // ===============================================================================
    // XCM Configuration (only when "xcm" feature is enabled)
    // ===============================================================================
    #[cfg(feature = "xcm")]
    mod xcm_config {
        use super::{
            Balances, PolkadotXcm, RuntimeCall, RuntimeOrigin, AllPalletsWithSystem,
            Weight, ConstU32, Everything, Nothing,
        };
        // Use frame_support's AccountId32, not XCM's AccountId32 junction
        use $crate::frame_support::sp_runtime::AccountId32 as RuntimeAccountId32;
        use $crate::xcm::latest::prelude::{
            Location, NetworkId, InteriorLocation, Junctions,
        };
        use $crate::xcm_builder::{
            AccountId32Aliases,
            AllowExplicitUnpaidExecutionFrom,
            AllowTopLevelPaidExecutionFrom,
            FixedWeightBounds,
            FrameTransactionalProcessor,
            SignedAccountId32AsNative,
            SignedToAccountId32,
            SovereignSignedViaLocation,
            TakeWeightCredit,
            WithComputedOrigin,
        };
        use $crate::pallet_xcm::XcmPassthrough;
        use $crate::frame_support::parameter_types;

        parameter_types! {
            pub const TokenLocation: Location = Location::here();
            pub const RelayNetwork: Option<NetworkId> = None;
            pub UniversalLocation: InteriorLocation = Junctions::Here;
            // One XCM operation is 1_000_000_000 weight - conservative estimate
            pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
            pub const MaxInstructions: u32 = 100;
            pub const MaxAssetsIntoHolding: u32 = 64;
        }

        /// Type for specifying how a `Location` can be converted into an `AccountId`.
        pub type LocationToAccountId = AccountId32Aliases<RelayNetwork, RuntimeAccountId32>;

        /// Means for transacting the native currency on this chain.
        #[allow(deprecated)]
        pub type LocalAssetTransactor = $crate::xcm_builder::CurrencyAdapter<
            Balances,
            $crate::xcm_builder::IsConcrete<TokenLocation>,
            LocationToAccountId,
            RuntimeAccountId32,
            (),
        >;

        /// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance.
        pub type XcmOriginToTransactDispatchOrigin = (
            SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
            SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
            XcmPassthrough<RuntimeOrigin>,
        );

        /// Barrier that allows everything - suitable for testing.
        pub type Barrier = (
            TakeWeightCredit,
            WithComputedOrigin<
                (
                    AllowTopLevelPaidExecutionFrom<Everything>,
                    AllowExplicitUnpaidExecutionFrom<Everything>,
                ),
                UniversalLocation,
                ConstU32<8>,
            >,
        );

        pub struct XcmConfig;
        impl $crate::xcm_executor::Config for XcmConfig {
            type RuntimeCall = RuntimeCall;
            type XcmSender = ();  // No sending for test runtime
            type XcmEventEmitter = PolkadotXcm;
            type AssetTransactor = LocalAssetTransactor;
            type OriginConverter = XcmOriginToTransactDispatchOrigin;
            type IsReserve = ();
            type IsTeleporter = ();
            type UniversalLocation = UniversalLocation;
            type Barrier = Barrier;
            type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
            type Trader = ();
            type ResponseHandler = PolkadotXcm;
            type AssetTrap = PolkadotXcm;
            type AssetClaims = PolkadotXcm;
            type SubscriptionService = PolkadotXcm;
            type PalletInstancesInfo = AllPalletsWithSystem;
            type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
            type AssetLocker = ();
            type AssetExchanger = ();
            type FeeManager = ();
            type MessageExporter = ();
            type UniversalAliases = Nothing;
            type CallDispatcher = RuntimeCall;
            type SafeCallFilter = Everything;
            type Aliasers = Nothing;
            type TransactionalProcessor = FrameTransactionalProcessor;
            type HrmpNewChannelOpenRequestHandler = ();
            type HrmpChannelAcceptedHandler = ();
            type HrmpChannelClosingHandler = ();
            type XcmRecorder = PolkadotXcm;
        }

        /// Local origins on this chain are allowed to dispatch XCM sends/executions.
        pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, RuntimeAccountId32, RelayNetwork>;
    }

    #[cfg(feature = "xcm")]
    pub use xcm_config::*;

    #[cfg(feature = "xcm")]
    impl $crate::pallet_xcm::Config for $runtime {
        type RuntimeEvent = RuntimeEvent;
        type SendXcmOrigin = $crate::xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
        type XcmRouter = ();  // No routing for test runtime
        type ExecuteXcmOrigin = $crate::xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
        type XcmExecuteFilter = Everything;
        type XcmExecutor = $crate::xcm_executor::XcmExecutor<XcmConfig>;
        type XcmTeleportFilter = Everything;
        type XcmReserveTransferFilter = Nothing;
        type Weigher = $crate::xcm_builder::FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
        type UniversalLocation = UniversalLocation;
        type RuntimeOrigin = RuntimeOrigin;
        type RuntimeCall = RuntimeCall;
        const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
        type AdvertisedXcmVersion = $crate::pallet_xcm::CurrentXcmVersion;
        type Currency = Balances;
        type CurrencyMatcher = ();
        type TrustedLockers = ();
        type SovereignAccountOf = LocationToAccountId;
        type MaxLockers = ConstU32<8>;
        type WeightInfo = $crate::pallet_xcm::TestWeightInfo;
        type AdminOrigin = $crate::frame_system::EnsureRoot<$crate::frame_support::sp_runtime::AccountId32>;
        type MaxRemoteLockConsumers = ConstU32<0>;
        type RemoteLockConsumerIdentifier = ();
        type AuthorizedAliasConsideration = $crate::frame_support::traits::Disabled;
    }

    // Configure `pallet-revive`
    type BalanceOf = <Balances as Currency<AccountId32>>::Balance;
    impl Convert<Weight, BalanceOf> for $runtime {
        fn convert(w: Weight) -> BalanceOf {
            w.ref_time().into()
        }
    }

    // Unit = the base number of indivisible units for balances
    const UNIT: Balance = 1_000_000_000_000;
    const MILLIUNIT: Balance = 1_000_000_000;

    const fn deposit(items: u32, bytes: u32) -> Balance {
        (items as Balance * UNIT + (bytes as Balance) * (5 * MILLIUNIT / 100)) / 10
    }

    parameter_types! {
        pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
        pub const MaxEthExtrinsicWeight: FixedU128 = FixedU128::from_rational(1,2);
        pub const DepositPerChildTrieItem: Balance = deposit(1, 0) / 100;
    }

    // Precompiles type alias - compose assets/xcm precompiles based on enabled features.
    // Precompiles type alias - conditionally includes XcmPrecompile when "xcm" feature is enabled
    #[cfg(not(feature = "xcm"))]
    pub type RevivePrecompiles = (
        $crate::pallet_assets_precompiles::ERC20<$runtime, $crate::pallet_assets_precompiles::InlineIdConfig<{ 0x0120 }>, TrustBackedAssetsInstance>,
    );

    #[cfg(feature = "xcm")]
    pub type RevivePrecompiles = (
        $crate::pallet_assets_precompiles::ERC20<$runtime, $crate::pallet_assets_precompiles::InlineIdConfig<{ 0x0120 }>, TrustBackedAssetsInstance>,
        $crate::pallet_xcm_precompiles::XcmPrecompile<$runtime>,
    );

    impl $crate::pallet_revive::Config for $runtime {
        type AddressMapper = $crate::pallet_revive::AccountId32Mapper<Self>;
        type ChainId = ConstU64<1>;
        type NativeToEthRatio = ConstU32<100_000_000>;
        type Time = Timestamp;
        type Balance = Balance;
        type Currency = Balances;
        type RuntimeEvent = RuntimeEvent;
        type RuntimeCall = RuntimeCall;
        type RuntimeOrigin = RuntimeOrigin;
        type DepositPerItem = ConstU128<1>;
        type DepositPerChildTrieItem = DepositPerChildTrieItem;
        type DepositPerByte = ConstU128<1>;
        type WeightInfo = ();
        type RuntimeMemory = ConstU32<{ 128 * 1024 * 1024 }>;
        type PVFMemory = ConstU32<{ 512 * 1024 * 1024 }>;
        type UnsafeUnstableInterface = ConstBool<true>;
        type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
        type RuntimeHoldReason = RuntimeHoldReason;
        type UploadOrigin = $crate::frame_system::EnsureSigned<Self::AccountId>;
        type InstantiateOrigin = $crate::frame_system::EnsureSigned<Self::AccountId>;
        type FindAuthor = ();
        type Precompiles = RevivePrecompiles;
        type AllowEVMBytecode = ConstBool<false>;
        type FeeInfo = ();
        type MaxEthExtrinsicWeight = MaxEthExtrinsicWeight;
        type DebugEnabled = ConstBool<false>;
    }

    /// Default initial balance for the default account.
    pub const INITIAL_BALANCE: u128 = 1_000_000_000_000_000;
    pub const DEFAULT_ACCOUNT: AccountId32 = AccountId32::new([1u8; 32]);

    pub struct $runtime_env {
        ext: $crate::TestExternalities,
    }

    impl ::std::default::Default for $runtime_env {
        fn default() -> Self {
            let ext = $crate::macros::BlockBuilder::<$runtime>::new_ext(vec![(
                DEFAULT_ACCOUNT,
                INITIAL_BALANCE,
            )]);
            Self { ext }
        }
    }

    // Implement `crate::RuntimeEnv` trait
    impl $crate::RuntimeEnv for $runtime_env {
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
#[cfg(not(feature = "xcm"))]
pub use construct_runtime::{
    $runtime_env, $runtime, Assets, AssetIdForTrustBackedAssets, Balances, Nfts,
    NftsCollectionId, NftsItemId, Revive, PalletInfo, RuntimeCall, RuntimeEvent, RuntimeHoldReason,
    RuntimeOrigin, System, Timestamp, TrustBackedAssetsInstance,
};

#[cfg(feature = "xcm")]
pub use construct_runtime::{
    $runtime_env, $runtime, Assets, AssetIdForTrustBackedAssets, Balances, Nfts,
    NftsCollectionId, NftsItemId, Revive, PalletInfo, PolkadotXcm, RuntimeCall, RuntimeEvent, RuntimeHoldReason,
    RuntimeOrigin, System, Timestamp, TrustBackedAssetsInstance,
};
    };
}

create_runtime!(DefaultRuntime);
