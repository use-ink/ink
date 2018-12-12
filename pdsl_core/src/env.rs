//! Externally defined and provded functionality.
//!
//! Refer to substrate SRML for more information.

/// Refer to substrate SRML contract module for more documentation.
pub mod c_abi {
	extern "C" {
		pub fn ext_create(
			init_code_ptr: u32,
			init_code_len: u32,
			gas: u64,
			value_ptr: u32,
			value_len: u32,
			input_data_ptr: u32,
			input_data_len: u32
		) -> u32;

		pub fn ext_call(
			callee_ptr: u32,
			callee_len: u32,
			gas: u64,
			value_ptr: u32,
			value_len: u32,
			input_data_ptr: u32,
			input_data_len: u32
		) -> u32;

		pub fn ext_caller();

		pub fn ext_set_storage(key_ptr: u32, value_non_null: u32, value_ptr: u32, value_len: u32);
		pub fn ext_get_storage(key_ptr: u32) -> u32;

		pub fn ext_scratch_size() -> u32;
		pub fn ext_scratch_copy(dest_ptr: u32, offset: u32, len: u32);

		pub fn ext_input_size() -> u32;
		pub fn ext_input_copy(dest_ptr: u32, offset: u32, len: u32);

		pub fn ext_return(data_ptr: u32, data_len: u32) -> !;
	}
}

/// The evironment API usable by SRML contracts.
pub trait Env {
	/// Returns the chain address of the caller.
	fn caller() -> Vec<u8>;
	/// Stores the given value under the given key.
	fn store(key: &[u8], value: &[u8]);
	/// Clears the value stored under the given key.
	fn clear(key: &[u8]);
	/// Loads data stored under the given key.
	fn load(key: &[u8]) -> Option<Vec<u8>>;
	/// Loads input data for contract execution.
	fn input() -> Vec<u8>;
	/// Returns from the contract execution with the given value.
	fn return_(value: &[u8]) -> !;
}

#[cfg(not(feature = "test-env"))]
mod default {
	use super::*;

	/// The default SRML contracts environment.
	pub struct DefaultEnv;

	impl Env for DefaultEnv {
		fn caller() -> Vec<u8> {
			unsafe { c_abi::ext_caller() };
			let size = unsafe { c_abi::ext_scratch_size() };
			let mut value = Vec::new();
			if size > 0 {
				value.resize(size as usize, 0);
				unsafe {
					c_abi::ext_scratch_copy(value.as_mut_ptr() as u32, 0, size);
				}
			}
			value
		}

		fn store(key: &[u8], value: &[u8]) {
			unsafe {
				c_abi::ext_set_storage(
					key.as_ptr() as u32,
					1,
					value.as_ptr() as u32,
					value.len() as u32
				);
			}
		}

		fn clear(key: &[u8]) {
			unsafe {
				c_abi::ext_set_storage(key.as_ptr() as u32, 0, 0, 0)
			}
		}

		fn load(key: &[u8]) -> Option<Vec<u8>> {
			const SUCCESS: u32 = 0;
			let result = unsafe { c_abi::ext_get_storage(key.as_ptr() as u32) };
			if result != SUCCESS {
				return None
			}
			let size = unsafe { c_abi::ext_scratch_size() };
			let mut value = Vec::new();
			if size > 0 {
				value.resize(size as usize, 0);
				unsafe {
					c_abi::ext_scratch_copy(value.as_mut_ptr() as u32, 0, size);
				}
			}
			Some(value)
		}

		fn input() -> Vec<u8> {
			let size = unsafe { c_abi::ext_input_size() };
			if size == 0 {
				Vec::new()
			} else {
				let mut buffer = Vec::new();
				buffer.resize(size as usize, 0);
				unsafe { c_abi::ext_input_copy(buffer.as_mut_ptr() as u32, 0, size); }
				buffer
			}
		}

		fn return_(data: &[u8]) -> ! {
			unsafe {
				c_abi::ext_return(data.as_ptr() as u32, data.len() as u32);
			}
		}
	}
}

#[cfg(feature = "test-env")]
mod test {
	use super::*;

	use lazy_static::lazy_static;
	use std::collections::HashMap;
	use std::sync::Mutex;

	lazy_static! {
		static ref CALLER: Mutex<Vec<u8>> = {
			Mutex::new(Vec::new())
		};
		static ref STORAGE: Mutex<HashMap<Vec<u8>, Vec<u8>>> = {
			Mutex::new(HashMap::new())
		};
		static ref INPUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());
		static ref EXPECTED_RETURN: Mutex<Vec<u8>> = Mutex::new(Vec::new());
	}

	/// Test environment for testing SRML contract off-chain.
	pub struct TestEnv;

	impl TestEnv {
		pub fn reset() {
			STORAGE.lock().unwrap().clear();
			INPUT.lock().unwrap().clear();
			EXPECTED_RETURN.lock().unwrap().clear();
		}

		pub fn expect_return(expected_bytes: &[u8]) {
			*EXPECTED_RETURN.lock().unwrap() = expected_bytes.to_vec();
		}

		pub fn set_input(input_bytes: &[u8]) {
			*INPUT.lock().unwrap() = input_bytes.to_vec();
		}
	}

	impl Env for TestEnv {
		fn caller() -> Vec<u8> {
			println!("TestEnv::caller()");
			CALLER.lock().unwrap().clone()
		}

		fn store(key: &[u8], value: &[u8]) {
			println!("TestEnv::store(\n\tkey: {:?},\n\tval: {:?}\n)", key, value);
			STORAGE.lock().unwrap().insert(key.to_vec(), value.to_vec());
		}

		fn clear(key: &[u8]) {
			println!("TestEnv::clear(\n\tkey: {:?}\n)", key);
			STORAGE.lock().unwrap().remove(key);
		}

		fn load(key: &[u8]) -> Option<Vec<u8>> {
			println!("TestEnv::load(\n\tkey: {:?}\n)", key);
			STORAGE.lock().unwrap().get(key).map(|loaded| loaded.to_vec())
		}

		fn input() -> Vec<u8> {
			INPUT.lock().unwrap().clone()
		}

		fn return_(data: &[u8]) -> ! {
			const SUCCESS: i32 = 0;
			const FAILURE: i32 = -1;
			*EXPECTED_RETURN.lock().unwrap() = data.to_vec();
			let exit_code = if data == EXPECTED_RETURN.lock().unwrap().as_slice() {
				SUCCESS
			} else {
				FAILURE
			};
			std::process::exit(exit_code)
		}
	}
}

#[cfg(not(feature = "test-env"))]
pub use self::default::DefaultEnv;

#[cfg(feature = "test-env")]
pub use self::test::TestEnv;

/// The environment implementation that is currently being used.
///
/// This may be either
/// - `DefaultEnv` for real contract storage
///   manipulation that may happen on-chain.
/// - `TestEnv` for emulating a contract environment
///   that can be inspected by the user and used
///   for testing contracts off-chain.
#[cfg(not(feature = "test-env"))]
pub type ContractEnv = self::default::DefaultEnv;

/// The environment implementation that is currently being used.
#[cfg(feature = "test-env")]
pub type ContractEnv = self::test::TestEnv;
