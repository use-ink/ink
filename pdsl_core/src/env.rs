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

		pub fn ext_set_storage(
			key_ptr: u32,
			value_non_null: u32,
			value_ptr: u32,
			value_len: u32
		);
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

	use std::cell::Cell;

	use std::collections::HashMap;
	use std::cell::RefCell;

	/// An entry in the storage of the test environment.
	///
	/// # Note
	///
	/// Additionally to its data it also stores the total
	/// number of reads and writes done to this entry.
	pub struct StorageEntry {
		/// The actual data that is stored in this storage entry.
		data: Vec<u8>,
		/// The number of reads to this storage entry.
		reads: Cell<u64>,
		/// The number of writes to this storage entry.
		writes: u64,
	}

	impl StorageEntry {
		/// Creates a new storage entry for the given data.
		pub fn new(data: Vec<u8>) -> Self {
			Self{data, reads: Cell::new(0), writes: 0}
		}

		/// Increases the read counter by one.
		fn inc_reads(&self) {
			self.reads.set(self.reads.get() + 1);
		}

		/// Increases the write counter by one.
		fn inc_writes(&mut self) {
			self.writes += 1;
		}

		/// Returns the number of reads for this storage entry.
		pub fn reads(&self) -> u64 {
			self.reads.get()
		}

		/// Returns the number of writes to this storage entry.
		pub fn writes(&self) -> u64 {
			self.writes
		}

		/// Returns the data stored in this storage entry.
		///
		/// # Note
		///
		/// Also bumps the read counter.
		pub fn read(&self) -> Vec<u8> {
			self.inc_reads();
			self.data.clone()
		}

		/// Writes the given data to this storage entry.
		///
		/// # Note
		///
		/// Also bumps the write counter.
		pub fn write(&mut self, new_data: Vec<u8>) {
			self.inc_writes();
			self.data = new_data;
		}
	}

	/// The data underlying to a test environment.
	pub struct TestEnvData {
		/// The storage entries.
		storage: HashMap<Vec<u8>, StorageEntry>,
		/// The caller address for the next contract invocation.
		///
		/// # Note
		///
		/// The current caller can be adjusted by `TestEnvData::set_caller`.
		caller: Vec<u8>,
		/// The input data for the next contract invocation.
		///
		/// # Note
		///
		/// The current input can be adjusted by `TestEnvData::set_input`.
		input: Vec<u8>,
		/// The expected return data of the next contract invocation.
		///
		/// # Note
		///
		/// This can be set by `TestEnvData::set_expected_return`.
		expected_return: Vec<u8>,
		/// The total number of reads from the storage.
		total_reads: Cell<u64>,
		/// The total number of writes to the storage.
		total_writes: u64,
	}

	impl Default for TestEnvData {
		fn default() -> Self {
			Self{
				storage: HashMap::new(),
				caller: Vec::new(),
				input: Vec::new(),
				expected_return: Vec::new(),
				total_reads: Cell::new(0),
				total_writes: 0,
			}
		}
	}

	impl TestEnvData {
		/// Resets `self` as if no contract execution happened so far.
		pub fn reset(&mut self) {
			self.storage.clear();
			self.caller.clear();
			self.input.clear();
			self.expected_return.clear();
			self.total_reads.set(0);
			self.total_writes = 0;
		}

		/// Increments the total number of reads from the storage.
		fn inc_total_reads(&self) {
			self.total_reads.set(self.total_reads.get() + 1)
		}

		/// Increments the total number of writes to the storage.
		fn inc_total_writes(&mut self) {
			self.total_writes += 1
		}

		/// Returns the total number of reads from the storage.
		pub fn total_reads(&self) -> u64 {
			self.total_reads.get()
		}

		/// Returns the total number of writes to the storage.
		pub fn total_writes(&self) -> u64 {
			self.total_writes
		}

		/// Returns the number of reads from the entry associated by the given key if any.
		pub fn reads_for(&self, key: &[u8]) -> Option<u64> {
			self.storage.get(key).map(|loaded| loaded.reads())
		}

		/// Returns the number of writes to the entry associated by the given key if any.
		pub fn writes_for(&self, key: &[u8]) -> Option<u64> {
			self.storage.get(key).map(|loaded| loaded.writes())
		}

		/// Sets the expected return data for the next contract invocation.
		pub fn set_expected_return(&mut self, expected_bytes: &[u8]) {
			self.expected_return = expected_bytes.to_vec();
		}

		/// Sets the caller address for the next contract invocation.
		pub fn set_caller(&mut self, new_caller: &[u8]) {
			self.caller = new_caller.to_vec();
		}

		/// Sets the input data for the next contract invocation.
		pub fn set_input(&mut self, input_bytes: &[u8]) {
			self.input = input_bytes.to_vec();
		}
	}

	use std::collections::hash_map::Entry;

	impl TestEnvData {
		/// The return code for successful contract invocations.
		///
		/// # Note
		///
		/// A contract invocation is successful if it returned the same data
		/// as was expected upon invocation.
		const SUCCESS: i32 = 0;
		/// The return code for unsuccessful contract invocations.
		///
		/// # Note
		///
		/// A contract invocation is unsuccessful if it did not return the
		/// same data as was expected upon invocation.
		const FAILURE: i32 = -1;

		/// Returns the caller of the contract invocation.
		pub fn caller(&self) -> Vec<u8> {
			self.caller.clone()
		}

		/// Stores the given value under the given key in the contract storage.
		pub fn store(&mut self, key: &[u8], value: &[u8]) {
			self.inc_total_writes();
			match self.storage.entry(key.to_vec()) {
				Entry::Occupied(mut occupied) => {
					occupied.get_mut().write(value.to_vec())
				}
				Entry::Vacant(vacant) => {
					vacant.insert(
						StorageEntry::new(value.to_vec())
					);
				}
			}
		}

		/// Clears the value under the given key in the contract storage.
		pub fn clear(&mut self, key: &[u8]) {
			// Storage clears count as storage write.
			self.inc_total_writes();
			self.storage.remove(key);
		}

		/// Returns the value under the given key in the contract storage if any.
		pub fn load(&self, key: &[u8]) -> Option<Vec<u8>> {
			self.inc_total_reads();
			self
				.storage
				.get(key)
				.map(|loaded| loaded.read())
		}

		/// Returns the input data for the contract invocation.
		pub fn input(&self) -> Vec<u8> {
			self.input.clone()
		}

		/// Returns the data to the internal caller.
		///
		/// # Note
		///
		/// This exits the current process (contract invocation)
		/// with a return code that is successful if the returned
		/// data matches the expected return data.
		pub fn return_(&self, data: &[u8]) -> ! {
			let expected_bytes = self.expected_return.clone();
			let exit_code = if expected_bytes == data {
				Self::SUCCESS
			} else {
				Self::FAILURE
			};
			std::process::exit(exit_code)
		}
	}

	thread_local! {
		/// The test environment data.
		///
		/// This needs to be thread local since tests are run
		/// in paralell by default which may lead to data races otherwise.
		pub static TEST_ENV_DATA: RefCell<TestEnvData> = {
			RefCell::new(TestEnvData::default())
		};
	}

	/// Test environment for testing SRML contract off-chain.
	pub struct TestEnv;

	impl TestEnv {
		/// Resets the test environment as if no contract execution happened so far.
		pub fn reset() {
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow_mut().reset()
			})
		}

		/// Returns the total number of reads from the storage.
		pub fn total_reads() -> u64 {
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow().total_reads()
			})
		}

		/// Returns the total number of writes to the storage.
		pub fn total_writes() -> u64 {
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow().total_writes()
			})
		}

		/// Returns the number of reads from the entry associated by the given key if any.
		pub fn reads_for(key: &[u8]) -> Option<u64> {
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow().reads_for(key)
			})
		}

		/// Returns the number of writes to the entry associated by the given key if any.
		pub fn writes_for(key: &[u8]) -> Option<u64> {
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow().writes_for(key)
			})
		}

		/// Sets the expected return data for the next contract invocation.
		pub fn set_expected_return(expected_bytes: &[u8]) {
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow_mut().set_expected_return(expected_bytes)
			})
		}

		/// Sets the caller address for the next contract invocation.
		pub fn set_caller(new_caller: &[u8]) {
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow_mut().set_caller(new_caller)
			})
		}

		/// Sets the input data for the next contract invocation.
		pub fn set_input(input_bytes: &[u8]) {
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow_mut().set_input(input_bytes)
			})
		}
	}

	impl Env for TestEnv {
		fn caller() -> Vec<u8> {
			println!("TestEnv::caller()");
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow().caller()
			})
		}

		fn store(key: &[u8], value: &[u8]) {
			println!("TestEnv::store(\n\tkey: {:?},\n\tval: {:?}\n)", key, value);
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow_mut().store(key, value)
			})
		}

		fn clear(key: &[u8]) {
			println!("TestEnv::clear(\n\tkey: {:?}\n)", key);
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow_mut().clear(key)
			})
		}

		fn load(key: &[u8]) -> Option<Vec<u8>> {
			println!("TestEnv::load(\n\tkey: {:?}\n)", key);
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow().load(key)
			})
		}

		fn input() -> Vec<u8> {
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow().input()
			})
		}

		fn return_(data: &[u8]) -> ! {
			TEST_ENV_DATA.with(|test_env| {
				test_env.borrow().return_(data)
			})
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
