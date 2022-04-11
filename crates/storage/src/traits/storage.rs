use core::marker::PhantomData;
use ink_primitives::StorageKey;

/// Returns storage key for the type
pub trait StorageKeyHolder {
    /// Storage key
    const KEY: StorageKey;
}

/// Helps to identify is the type is atomic or not
pub trait AtomicStatus {
    /// Atomic status
    const IS_ATOMIC: bool;
    /// Atomic status of inner type if it exists
    const INNER_IS_ATOMIC: bool;
}

/// Returns the type that should be used for storing the value
pub trait StorageType<
    const KEY: StorageKey,
    Salt: StorageKeyHolder,
    const IS_ATOMIC: bool,
>
{
    /// Type with storage key inside
    type Type: AtomicStatus;
}

/// That key type means that the storage key should be calculated automatically.
pub struct AutoKey;

impl StorageKeyHolder for AutoKey {
    const KEY: StorageKey = 0;
}

/// That key type specifies the storage key.
pub struct ManualKey<const KEY: StorageKey, Salt: StorageKeyHolder = ()>(
    PhantomData<fn() -> Salt>,
);

impl<const KEY: StorageKey, Salt: StorageKeyHolder> StorageKeyHolder
    for ManualKey<KEY, Salt>
{
    // TODO: Use XoR here or better to calculate const hash during compilation?
    const KEY: StorageKey = KEY ^ Salt::KEY;
}
