//! A stash collection.
//!
//! Provides O(1) random insertion, deletion and access of its elements.
//!
//! ## Guarantees and non-guarantees:
//!
//! 1. `Stash` is deterministic and keys do not depend on the inserted values.
//!    This means you can update two stashes in tandem and get the same keys
//!    back. This could be useful for, e.g., primary/secondary replication.
//! 2. Keys will always be less than the maximum number of items that have ever
//!    been present in the `Stash` at any single point in time. In other words,
//!    if you never store more than `n` items in a `Stash`, the stash will only
//!    assign keys less than `n`. You can take advantage of this guarantee to
//!    truncate the key from a `usize` to some smaller type.
//! 3. Except the guarantees noted above, you can assume nothing about key
//!    assignment or iteration order. They can change at any time.

#[cfg(all(test, feature = "test-env"))]
mod tests;

mod stash;

pub use self::stash::{
	Stash,
};
