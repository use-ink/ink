use core::{
    fmt::Debug,
    marker::PhantomData,
};
use ink_primitives::StorageKey;

/// Returns storage key for the type
pub trait StorageKeyHolder {
    /// Storage key
    const KEY: StorageKey;
}

/// Helps to identify is the type is atomic or not. Type is not atomic if it requires
/// a separate storage cell.
pub trait AtomicStatus {
    /// Atomic status of the type.
    const IS_ATOMIC: bool;
}

/// `AtomicGuard<true>` is implemented for all primitive types and atomic structures.
/// It can be used to add requirement for the generic to be atomic.
pub trait AtomicGuard<const IS_ATOMIC: bool> {}

/// Returns the type that should be used for storing the value
pub trait StorageType<Salt: StorageKeyHolder> {
    /// Type with storage key inside
    type Type: AtomicStatus;
}

/// That key type means that the storage key should be calculated automatically.
#[derive(Default, Copy, Clone, PartialEq, PartialOrd, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct AutoKey;

impl StorageKeyHolder for AutoKey {
    const KEY: StorageKey = 0;
}

/// That key type specifies the storage key.
#[derive(Default, Copy, Clone, PartialEq, PartialOrd, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ManualKey<const KEY: StorageKey, Salt: StorageKeyHolder = ()>(
    PhantomData<fn() -> Salt>,
);

impl<const KEY: StorageKey, Salt: StorageKeyHolder> StorageKeyHolder
    for ManualKey<KEY, Salt>
{
    // TODO: Use XoR here or better to calculate const hash during compilation?
    const KEY: StorageKey = KEY ^ Salt::KEY;
}
