// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use std::convert::TryFrom;
use super::*;
use crate::{
    env::srml,
    memory::collections::hash_map::{
        Entry,
        HashMap,
    },
    storage::Key,
};
use core::cell::{
    Cell,
    RefCell,
};

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
        Self {
            data,
            reads: Cell::new(0),
            writes: 0,
        }
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
    storage: HashMap<Key, StorageEntry>,
    /// The caller address for the next contract invocation.
    ///
    /// # Note
    ///
    /// The current caller can be adjusted by `TestEnvData::set_caller`.
    caller: srml::Address,
    /// The input data for the next contract invocation.
    ///
    /// # Note
    ///
    /// The current input can be adjusted by `TestEnvData::set_input`.
    input: Vec<u8>,
    /// The random seed for the next contract invocation.
    ///
    ///  # Note
    ///
    /// The current random seed can be adjusted by `TestEnvData::set_random_seed`.
    random_seed: srml::Hash,
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
        Self {
            storage: HashMap::new(),
            caller: srml::Address([0x0; 32]),
            input: Vec::new(),
            random_seed: srml::Hash([0x0; 32]),
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
        self.caller = srml::Address([0; 32]);
        self.input.clear();
        self.random_seed = srml::Hash([0; 32]);
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
    pub fn reads_for(&self, key: Key) -> Option<u64> {
        self.storage.get(&key).map(|loaded| loaded.reads())
    }

    /// Returns the number of writes to the entry associated by the given key if any.
    pub fn writes_for(&self, key: Key) -> Option<u64> {
        self.storage.get(&key).map(|loaded| loaded.writes())
    }

    /// Sets the expected return data for the next contract invocation.
    pub fn set_expected_return(&mut self, expected_bytes: &[u8]) {
        self.expected_return = expected_bytes.to_vec();
    }

    /// Sets the caller address for the next contract invocation.
    pub fn set_caller(&mut self, new_caller: srml::Address) {
        self.caller = new_caller;
    }

    /// Sets the input data for the next contract invocation.
    pub fn set_input(&mut self, input_bytes: &[u8]) {
        self.input = input_bytes.to_vec();
    }

    /// Sets the random seed for the next contract invocation.
    pub fn set_random_seed(&mut self, random_seed_hash: srml::Hash) {
        self.random_seed = random_seed_hash;
    }
}

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
    pub fn caller(&self) -> srml::Address {
        self.caller
    }

    /// Stores the given value under the given key in the contract storage.
    pub fn store(&mut self, key: Key, value: &[u8]) {
        self.inc_total_writes();
        match self.storage.entry(key) {
            Entry::Occupied(mut occupied) => occupied.get_mut().write(value.to_vec()),
            Entry::Vacant(vacant) => {
                vacant.insert(StorageEntry::new(value.to_vec()));
            }
        }
    }

    /// Clears the value under the given key in the contract storage.
    pub fn clear(&mut self, key: Key) {
        // Storage clears count as storage write.
        self.inc_total_writes();
        self.storage.remove(&key);
    }

    /// Returns the value under the given key in the contract storage if any.
    pub fn load(&self, key: Key) -> Option<Vec<u8>> {
        self.inc_total_reads();
        self.storage.get(&key).map(|loaded| loaded.read())
    }

    /// Returns the input data for the contract invocation.
    pub fn input(&self) -> Vec<u8> {
        self.input.clone()
    }

    /// Returns the random seed for the contract invocation.
    pub fn random_seed(&self) -> srml::Hash {
        self.random_seed
    }

    /// Returns the data to the internal caller.
    ///
    /// # Note
    ///
    /// This exits the current process (contract invocation)
    /// with a return code that is successful if the returned
    /// data matches the expected return data.
    pub fn r#return(&self, data: &[u8]) -> ! {
        let expected_bytes = self.expected_return.clone();
        let exit_code = if expected_bytes == data {
            Self::SUCCESS
        } else {
            Self::FAILURE
        };
        std::process::exit(exit_code)
    }

    /// Prints the given content.
    pub fn println(&self, content: &str) {
        println!("{}", content)
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
        TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().reset())
    }

    /// Returns the total number of reads from the storage.
    pub fn total_reads() -> u64 {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().total_reads())
    }

    /// Returns the total number of writes to the storage.
    pub fn total_writes() -> u64 {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().total_writes())
    }

    /// Returns the number of reads from the entry associated by the given key if any.
    pub fn reads_for(key: Key) -> Option<u64> {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().reads_for(key))
    }

    /// Returns the number of writes to the entry associated by the given key if any.
    pub fn writes_for(key: Key) -> Option<u64> {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().writes_for(key))
    }

    /// Sets the expected return data for the next contract invocation.
    pub fn set_expected_return(expected_bytes: &[u8]) {
        TEST_ENV_DATA
            .with(|test_env| test_env.borrow_mut().set_expected_return(expected_bytes))
    }

    /// Sets the caller address for the next contract invocation.
    pub fn set_caller(new_caller: srml::Address) {
        TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().set_caller(new_caller))
    }

    /// Sets the input data for the next contract invocation.
    pub fn set_input(input_bytes: &[u8]) {
        TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().set_input(input_bytes))
    }

    /// Sets the random seed for the next contract invocation.
    pub fn set_random_seed(random_seed_bytes: srml::Hash) {
        TEST_ENV_DATA
            .with(|test_env| test_env.borrow_mut().set_random_seed(random_seed_bytes))
    }
}

const TEST_ENV_LOG_TARGET: &str = "test-env";

impl EnvTypes for TestEnv {
    type Address = srml::Address;
    type Balance = srml::Balance;
    type Hash = srml::Hash;
}

impl EnvStorage for TestEnv {
    unsafe fn store(key: Key, value: &[u8]) {
        log::debug!(
            target: TEST_ENV_LOG_TARGET,
            "TestEnv::store(\n\tkey: {:?},\n\tval: {:?}\n)",
            key,
            value,
        );
        TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().store(key, value))
    }

    unsafe fn clear(key: Key) {
        log::debug!(
            target: TEST_ENV_LOG_TARGET,
            "TestEnv::clear(\n\tkey: {:?}\n)",
            key,
        );
        TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().clear(key))
    }

    unsafe fn load(key: Key) -> Option<Vec<u8>> {
        log::debug!(
            target: TEST_ENV_LOG_TARGET,
            "TestEnv::load(\n\tkey: {:?}\n)",
            key,
        );
        TEST_ENV_DATA.with(|test_env| test_env.borrow().load(key))
    }
}

impl Env for TestEnv
where
    <Self as EnvTypes>::Address: for<'a> TryFrom<&'a [u8]>,
    <Self as EnvTypes>::Hash: for<'a> TryFrom<&'a [u8]>,

{
    fn caller() -> <Self as EnvTypes>::Address {
        log::debug!(target: TEST_ENV_LOG_TARGET, "TestEnv::caller()");
        TEST_ENV_DATA.with(|test_env| test_env.borrow().caller())
    }

    fn input() -> Vec<u8> {
        log::debug!(target: TEST_ENV_LOG_TARGET, "TestEnv::input()",);
        TEST_ENV_DATA.with(|test_env| test_env.borrow().input())
    }

    fn random_seed() -> <Self as EnvTypes>::Hash {
        log::debug!(target: TEST_ENV_LOG_TARGET, "TestEnv::random_seed()",);
        TEST_ENV_DATA.with(|test_env| test_env.borrow().random_seed())
    }

    unsafe fn r#return(data: &[u8]) -> ! {
        log::debug!(
            target: TEST_ENV_LOG_TARGET,
            "TestEnv::return_(\n\tdata: {:?}\n)",
            data,
        );
        TEST_ENV_DATA.with(|test_env| test_env.borrow().r#return(data))
    }

    fn println(content: &str) {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().println(content))
    }
}
