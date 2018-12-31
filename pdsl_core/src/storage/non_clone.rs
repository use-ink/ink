use parity_codec_derive::{Encode, Decode};

use core::marker::PhantomData;

/// Marks types as non-`Copy` and non-`Clone`.
///
/// # Note
///
/// - This marker type is zero-sized for all `T`.
/// - This serves as documentation for crate maintainers
///   as well as utility to enforce this behaviour for types
///   that are using it.
/// - Especially for `Cell` types it is important to make them
///   non-`Copy` and non-`Clone` since that would violate their
///   ownership guarantees over their contract storage slot.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Encode, Decode)]
pub struct NonCloneMarker<T>(PhantomData<T>);

impl<T> Default for NonCloneMarker<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}
