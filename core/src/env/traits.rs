use crate::{
    memory::vec::Vec,
    storage::Key,
};
use parity_codec::Codec;

/// The environmental types usable by contracts defined with pDSL.
pub trait EnvTypes {
    /// The type of an address.
    type Address: Codec + PartialEq + Eq;
    /// The type of balances.
    type Balance: Codec;
}

/// Types implementing this can act as contract storage.
pub trait EnvStorage {
    /// Stores the given value under the given key.
	///
	/// # Safety
	///
	/// This operation is unsafe becaues it does not check for key integrity.
	/// Users can compare this operation with a raw pointer dereferencing in Rust.
    unsafe fn store(key: Key, value: &[u8]);

    /// Clears the value stored under the given key.
	///
	/// # Safety
	///
	/// This operation is unsafe becaues it does not check for key integrity.
	/// Users can compare this operation with a raw pointer dereferencing in Rust.
    unsafe fn clear(key: Key);

    /// Loads data stored under the given key.
	///
	/// # Safety
	///
	/// This operation is unsafe becaues it does not check for key integrity.
	/// Users can compare this operation with a raw pointer dereferencing in Rust.
    unsafe fn load(key: Key) -> Option<Vec<u8>>;
}

/// The evironment API usable by contracts defined with pDSL.
pub trait Env: EnvTypes + EnvStorage {
    /// Returns the chain address of the caller.
    fn caller() -> <Self as EnvTypes>::Address;

    /// Loads input data for contract execution.
    fn input() -> Vec<u8>;

    /// Returns from the contract execution with the given value.
	///
	/// # Safety
	///
	/// The external callers rely on the correct type of the encoded
	/// returned value. This API is unsafe because it does not provide
	/// guarantees on its own to always encode the expected type.
    unsafe fn r#return(value: &[u8]) -> !;
}
