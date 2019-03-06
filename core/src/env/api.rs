use super::ContractEnv;
use crate::{
	storage::Key,
	memory::vec::Vec,
	env::{
		Env as _,
		EnvStorage as _,
		EnvTypes,
	},
};

/// The environmental address type.
pub type Address = <ContractEnv as EnvTypes>::Address;

/// The environmental balance type.
pub type Balance = <ContractEnv as EnvTypes>::Balance;

/// Returns the address of the caller of the current smart contract execution.
pub fn caller() -> Address {
	ContractEnv::caller()
}

/// Returns the uninterpreted input data of the current smart contract execution.
pub fn input() -> Vec<u8> {
	ContractEnv::input()
}

/// Returns the current smart contract exection back to the caller
/// and return the given encoded value.
///
/// # Safety
///
/// External callers rely on the correct type of the encoded returned value.
/// This operation is unsafe because it does not provide guarantees on its
/// own to always encode the expected type.
pub unsafe fn r#return<T>(value: T) -> !
where
	T: parity_codec::Encode,
{
	ContractEnv::r#return(&mut &value.encode()[..])
}

/// Stores the given value under the specified key in the contract storage.
///
/// # Safety
///
/// This operation is unsafe becaues it does not check for key integrity.
/// Users can compare this operation with a raw pointer dereferencing in Rust.
pub unsafe fn store(key: Key, value: &[u8]) {
	ContractEnv::store(key, value)
}

/// Clears the data stored at the given key from the contract storage.
///
/// # Safety
///
/// This operation is unsafe becaues it does not check for key integrity.
/// Users can compare this operation with a raw pointer dereferencing in Rust.
pub unsafe fn clear(key: Key) {
	ContractEnv::clear(key)
}

/// Loads the data stored at the given key from the contract storage.
///
/// # Safety
///
/// This operation is unsafe becaues it does not check for key integrity.
/// Users can compare this operation with a raw pointer dereferencing in Rust.
pub unsafe fn load(key: Key) -> Option<Vec<u8>> {
	ContractEnv::load(key)
}
