// Copyright 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::cell::{
    Cell,
    RefCell,
};
use std::marker::PhantomData;

use scale::{
    Decode,
    Encode,
};

use super::*;
use crate::{
    env::{
        CallError,
        EnvTypes,
    },
    memory::collections::hash_map::{
        Entry,
        HashMap,
    },
    storage::Key,
};

/// A wrapper for the generic bytearray used for data in contract events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventData {
    topics: Vec<Vec<u8>>,
    data: Vec<u8>,
}

/// Raw recorded data of smart contract creates and instantiations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawCreateData {
    code_hash: Vec<u8>,
    gas_limit: u64,
    value: Vec<u8>,
    input_data: Vec<u8>,
}

/// Decoded recorded data of smart contract creates and instantiations.
pub struct CreateData<E>
where
    E: EnvTypes,
{
    pub code_hash: E::Hash,
    pub gas_limit: u64,
    pub value: E::Balance,
    pub input_data: Vec<u8>,
}

impl<E> From<RawCreateData> for CreateData<E>
where
    E: EnvTypes,
{
    fn from(raw: RawCreateData) -> Self {
        Self {
            code_hash: Decode::decode(&mut &raw.code_hash[..])
                .expect("encountered invalid encoded code hash"),
            gas_limit: raw.gas_limit,
            value: Decode::decode(&mut &raw.value[..])
                .expect("encountered invalid encoded value"),
            input_data: raw.input_data,
        }
    }
}

impl EventData {
    /// Returns the uninterpreted bytes of the emitted event.
    fn data_as_bytes(&self) -> &[u8] {
        self.data.as_slice()
    }
}

/// Emulates the data given to remote smart contract call instructions.
pub struct RawCallData {
    pub callee: Vec<u8>,
    pub gas: u64,
    pub value: Vec<u8>,
    pub input_data: Vec<u8>,
}

/// Decoded call data of recorded external calls.
pub struct CallData<E>
where
    E: EnvTypes,
{
    pub callee: E::AccountId,
    pub gas: u64,
    pub value: E::Balance,
    pub input_data: Vec<u8>,
}

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
    /// The address of the contract.
    ///
    /// # Note
    ///
    /// The current address can be adjusted by `TestEnvData::set_address`.
    address: Vec<u8>,
    /// The balance of the contract.
    ///
    /// # Note
    ///
    /// The current balance can be adjusted by `TestEnvData::set_balance`.
    balance: Vec<u8>,
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
    /// The random seed for the next contract invocation.
    ///
    ///  # Note
    ///
    /// The current random seed can be adjusted by `TestEnvData::set_random_seed`.
    random_seed: Vec<u8>,
    /// The timestamp for the next contract invocation.
    ///
    /// # Note
    ///
    /// The current timestamp can be adjusted by `TestEnvData::set_now`.
    now: Vec<u8>,
    /// The current block number for the next contract invocation.
    ///
    /// # Note
    ///
    /// The current current block number can be adjusted by `TestEnvData::set_block_number`.
    block_number: Vec<u8>,
    /// The total number of reads from the storage.
    total_reads: Cell<u64>,
    /// The total number of writes to the storage.
    total_writes: u64,
    /// Deposited events of the contract invocation.
    events: Vec<EventData>,
    /// Calls dispatched to the runtime
    dispatched_calls: Vec<Vec<u8>>,
    /// The current gas price.
    gas_price: Vec<u8>,
    /// The remaining gas.
    gas_left: Vec<u8>,
    /// The total transferred value.
    value_transferred: Vec<u8>,
    /// The recorded external calls.
    calls: Vec<RawCallData>,
    /// The expected return data of the next external call.
    call_return: Vec<u8>,
    /// Returned data.
    return_data: Vec<u8>,
    /// Recorded smart contract instantiations.
    creates: Vec<RawCreateData>,
    /// The address of the next instantiated smart contract.
    next_create_address: Vec<u8>,
}

impl Default for TestEnvData {
    fn default() -> Self {
        Self {
            address: Vec::new(),
            storage: HashMap::new(),
            balance: Vec::new(),
            caller: Vec::new(),
            input: Vec::new(),
            random_seed: Vec::new(),
            now: Vec::new(),
            block_number: Vec::new(),
            total_reads: Cell::new(0),
            total_writes: 0,
            events: Vec::new(),
            gas_price: Vec::new(),
            gas_left: Vec::new(),
            value_transferred: Vec::new(),
            dispatched_calls: Vec::new(),
            calls: Vec::new(),
            call_return: Vec::new(),
            return_data: Vec::new(),
            creates: Vec::new(),
            next_create_address: Vec::new(),
        }
    }
}

impl TestEnvData {
    /// Resets `self` as if no contract execution happened so far.
    pub fn reset(&mut self) {
        self.address.clear();
        self.balance.clear();
        self.storage.clear();
        self.caller.clear();
        self.input.clear();
        self.random_seed.clear();
        self.now.clear();
        self.block_number.clear();
        self.total_reads.set(0);
        self.total_writes = 0;
        self.events.clear();
        self.dispatched_calls.clear();
        self.calls.clear();
        self.call_return.clear();
        self.return_data.clear();
        self.creates.clear();
        self.next_create_address.clear();
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

    /// Sets the contract address for the next contract invocation.
    pub fn set_address(&mut self, new_address: Vec<u8>) {
        self.address = new_address;
    }

    /// Sets the contract balance for the next contract invocation.
    pub fn set_balance(&mut self, new_balance: Vec<u8>) {
        self.balance = new_balance;
    }

    /// Sets the caller address for the next contract invocation.
    pub fn set_caller(&mut self, new_caller: Vec<u8>) {
        self.caller = new_caller;
    }

    /// Sets the input data for the next contract invocation.
    pub fn set_input(&mut self, input_bytes: Vec<u8>) {
        self.input = input_bytes;
    }

    /// Appends new event data to the end of the bytearray.
    pub fn add_event(&mut self, topics: &[Vec<u8>], event_data: &[u8]) {
        let new_event = EventData {
            topics: topics.to_vec(),
            data: event_data.to_vec(),
        };
        self.events.push(new_event);
    }

    /// Appends a dispatched call to the runtime
    pub fn add_dispatched_call(&mut self, call: &[u8]) {
        self.dispatched_calls.push(call.to_vec());
    }

    /// Sets the random seed for the next contract invocation.
    pub fn set_random_seed(&mut self, random_seed_hash: Vec<u8>) {
        self.random_seed = random_seed_hash.to_vec();
    }

    /// Sets the timestamp for the next contract invocation.
    pub fn set_now(&mut self, timestamp: Vec<u8>) {
        self.now = timestamp;
    }

    /// Sets the current block number for the next contract invocation.
    pub fn set_block_number(&mut self, block_number: Vec<u8>) {
        self.block_number = block_number;
    }

    /// Returns an iterator over all emitted events.
    pub fn emitted_events(&self) -> impl DoubleEndedIterator<Item = &[u8]> {
        self.events
            .iter()
            .map(|event_data| event_data.data_as_bytes())
    }

    /// Returns an iterator over all dispatched calls
    pub fn dispatched_calls(&self) -> impl DoubleEndedIterator<Item = &[u8]> {
        self.dispatched_calls.iter().map(Vec::as_slice)
    }

    /// Records a new external call.
    pub fn add_call(&mut self, callee: &[u8], gas: u64, value: &[u8], input_data: &[u8]) {
        let new_call = RawCallData {
            callee: callee.to_vec(),
            gas,
            value: value.to_vec(),
            input_data: input_data.to_vec(),
        };
        self.calls.push(new_call);
    }

    /// Returns an iterator over all recorded external calls.
    pub fn external_calls(&self) -> impl DoubleEndedIterator<Item = &RawCallData> {
        self.calls.iter()
    }

    /// Set the expected return data of the next external call.
    pub fn set_return_data(&mut self, return_data: &[u8]) {
        self.return_data = return_data.to_vec();
    }

    /// Returns the latest returned data.
    pub fn returned_data(&self) -> &[u8] {
        &self.return_data
    }

    /// Records a new smart contract instantiation.
    pub fn add_create(
        &mut self,
        code_hash: &[u8],
        gas_limit: u64,
        value: &[u8],
        input_data: &[u8],
    ) {
        let new_create = RawCreateData {
            code_hash: code_hash.to_vec(),
            gas_limit,
            value: value.to_vec(),
            input_data: input_data.to_vec(),
        };
        self.creates.push(new_create);
    }

    /// Returns an iterator over all recorded smart contract instantiations.
    pub fn creates(&self) -> impl DoubleEndedIterator<Item = &RawCreateData> {
        self.creates.iter()
    }

    /// Sets the address of the next instantiated smart contract.
    pub fn set_next_create_address(&mut self, account_id: &[u8]) {
        self.next_create_address = account_id.to_vec();
    }
}

impl TestEnvData {
    pub fn address(&self) -> Vec<u8> {
        self.address.clone()
    }

    pub fn balance(&self) -> Vec<u8> {
        self.balance.clone()
    }

    pub fn caller(&self) -> Vec<u8> {
        self.caller.clone()
    }

    pub fn store(&mut self, key: Key, value: &[u8]) {
        self.inc_total_writes();
        match self.storage.entry(key) {
            Entry::Occupied(mut occupied) => occupied.get_mut().write(value.to_vec()),
            Entry::Vacant(vacant) => {
                vacant.insert(StorageEntry::new(value.to_vec()));
            }
        }
    }

    pub fn clear(&mut self, key: Key) {
        // Storage clears count as storage write.
        self.inc_total_writes();
        self.storage.remove(&key);
    }

    pub fn load(&self, key: Key) -> Option<Vec<u8>> {
        self.inc_total_reads();
        self.storage.get(&key).map(|loaded| loaded.read())
    }

    pub fn input(&self) -> Vec<u8> {
        self.input.clone()
    }

    pub fn random_seed(&self) -> Vec<u8> {
        self.random_seed.clone()
    }

    pub fn now(&self) -> Vec<u8> {
        self.now.clone()
    }

    pub fn block_number(&self) -> Vec<u8> {
        self.block_number.clone()
    }

    pub fn gas_price(&self) -> Vec<u8> {
        self.gas_price.clone()
    }

    pub fn gas_left(&self) -> Vec<u8> {
        self.gas_left.clone()
    }

    pub fn value_transferred(&self) -> Vec<u8> {
        self.value_transferred.clone()
    }

    pub fn return_data(&mut self, data: &[u8]) {
        self.return_data = data.to_vec();
    }

    pub fn println(&self, content: &str) {
        println!("{}", content)
    }

    pub fn deposit_raw_event(&mut self, topics: &[Vec<u8>], data: &[u8]) {
        self.add_event(topics, data);
    }

    pub fn dispatch_call(&mut self, call: &[u8]) {
        self.add_dispatched_call(call);
    }

    pub fn call(
        &mut self,
        callee: &[u8],
        gas: u64,
        value: &[u8],
        input_data: &[u8],
    ) -> Vec<u8> {
        self.add_call(callee, gas, value, input_data);
        self.call_return.clone()
    }

    pub fn create(
        &mut self,
        code_hash: &[u8],
        gas_limit: u64,
        value: &[u8],
        input_data: &[u8],
    ) -> Vec<u8> {
        self.add_create(code_hash, gas_limit, value, input_data);
        self.next_create_address.clone()
    }
}

thread_local! {
    /// The test environment data.
    ///
    /// This needs to be thread local since tests are run
    /// in parallel by default which may lead to data races otherwise.
    pub static TEST_ENV_DATA: RefCell<TestEnvData> = {
        RefCell::new(TestEnvData::default())
    };
}

/// Test environment for testing SRML contract off-chain.
pub struct TestEnv<T> {
    marker: PhantomData<fn() -> T>,
}

macro_rules! impl_env_setters_for_test_env {
    ( $( ($fn_name:ident, $name:ident, $ty:ty) ),* ) => {
        $(
            pub fn $fn_name($name: $ty) {
                TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().$fn_name($name.encode()))
            }
        )*
    }
}

impl<T> TestEnv<T>
where
    T: EnvTypes,
{
    /// Resets the test environment as if no contract execution happened so far.
    pub fn reset() {
        TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().reset())
    }

    /// Returns the number of reads from the entry associated by the given key if any.
    pub fn reads_for(key: Key) -> Option<u64> {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().reads_for(key))
    }

    /// Returns the number of writes to the entry associated by the given key if any.
    pub fn writes_for(key: Key) -> Option<u64> {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().writes_for(key))
    }

    /// Returns the latest returned data.
    pub fn returned_data() -> Vec<u8> {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().returned_data().to_vec())
    }

    /// Sets the input data for the next contract invocation.
    pub fn set_input(input_bytes: &[u8]) {
        TEST_ENV_DATA
            .with(|test_env| test_env.borrow_mut().set_input(input_bytes.to_vec()))
    }

    /// Sets the expected return data for the next external call.
    pub fn set_return_data(expected_return_data: &[u8]) {
        TEST_ENV_DATA
            .with(|test_env| test_env.borrow_mut().set_return_data(expected_return_data))
    }

    /// Sets the address of the next instantiated smart contract.
    pub fn set_next_create_address(account_id: T::AccountId) {
        TEST_ENV_DATA.with(|test_env| {
            test_env
                .borrow_mut()
                .set_next_create_address(&account_id.encode())
        })
    }

    impl_env_setters_for_test_env!(
        (set_address, address, T::AccountId),
        (set_balance, balance, T::Balance),
        (set_caller, caller, T::AccountId),
        (set_random_seed, random_seed, T::Hash),
        (set_now, now, T::Moment),
        (set_block_number, block_number, T::BlockNumber)
    );

    /// Returns an iterator over all emitted events.
    pub fn emitted_events() -> impl DoubleEndedIterator<Item = Vec<u8>> {
        TEST_ENV_DATA.with(|test_env| {
            test_env
                .borrow()
                .emitted_events()
                .map(|event_bytes| event_bytes.to_vec())
                .collect::<Vec<_>>()
                .into_iter()
        })
    }

    /// Returns an iterator over all emitted events.
    pub fn external_calls() -> impl DoubleEndedIterator<Item = CallData<T>> {
        TEST_ENV_DATA.with(|test_env| {
            test_env
                .borrow()
                .external_calls()
                .map(|raw_call_data| {
                    CallData {
                        callee: Decode::decode(&mut &raw_call_data.callee[..])
                            .expect("invalid encoded callee"),
                        gas: raw_call_data.gas,
                        value: Decode::decode(&mut &raw_call_data.value[..])
                            .expect("invalid encoded value"),
                        input_data: raw_call_data.input_data.clone(),
                    }
                })
                .collect::<Vec<_>>()
                .into_iter()
        })
    }

    /// Returns an iterator over all recorded smart contract instantiations.
    pub fn creates() -> impl DoubleEndedIterator<Item = CreateData<T>> {
        TEST_ENV_DATA.with(|test_env| {
            test_env
                .borrow()
                .creates()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into_iter()
        })
    }

    /// Returns an iterator over all dispatched calls.
    pub fn dispatched_calls() -> impl DoubleEndedIterator<Item = T::Call> {
        TEST_ENV_DATA.with(|test_env| {
            test_env
                .borrow()
                .dispatched_calls()
                .map(|call| Decode::decode(&mut &call[..]).expect("Valid encoded Call"))
                .collect::<Vec<_>>()
                .into_iter()
        })
    }
}

macro_rules! impl_env_getters_for_test_env {
    ( $( ($fn_name:ident, $ret_name:ty) ),* ) => {
        $(
            fn $fn_name() -> $ret_name {
                TEST_ENV_DATA.with(|test_env| Decode::decode(&mut &test_env.borrow().$fn_name()[..])
                    .expect("environment instances are assumed to be correctly encoded"))
            }
        )*
    }
}

impl<T> EnvTypes for TestEnv<T>
where
    T: EnvTypes,
{
    type AccountId = <T as EnvTypes>::AccountId;
    type Balance = <T as EnvTypes>::Balance;
    type Hash = <T as EnvTypes>::Hash;
    type Moment = <T as EnvTypes>::Moment;
    type BlockNumber = <T as EnvTypes>::BlockNumber;
    type Call = <T as EnvTypes>::Call;
}

impl<T> Env for TestEnv<T>
where
    T: EnvTypes,
{
    impl_env_getters_for_test_env!(
        (address, T::AccountId),
        (balance, T::Balance),
        (caller, T::AccountId),
        (input, Vec<u8>),
        (random_seed, T::Hash),
        (now, T::Moment),
        (block_number, T::BlockNumber),
        (gas_price, T::Balance),
        (gas_left, T::Balance),
        (value_transferred, T::Balance)
    );

    fn return_data(data: &[u8]) {
        TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().return_data(data))
    }

    fn println(content: &str) {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().println(content))
    }

    fn deposit_raw_event(topics: &[T::Hash], data: &[u8]) {
        TEST_ENV_DATA.with(|test_env| {
            let topics = topics.iter().map(Encode::encode).collect::<Vec<_>>();
            test_env.borrow_mut().deposit_raw_event(&topics, data)
        })
    }

    fn dispatch_raw_call(data: &[u8]) {
        TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().dispatch_call(data))
    }

    fn call_invoke(
        callee: T::AccountId,
        gas: u64,
        value: T::Balance,
        input_data: &[u8],
    ) -> Result<(), CallError> {
        let callee = &(callee.encode())[..];
        let value = &(value.encode())[..];
        let _return_data = TEST_ENV_DATA
            .with(|test_env| test_env.borrow_mut().call(callee, gas, value, input_data));
        Ok(())
    }

    fn call_evaluate<U: Decode>(
        callee: T::AccountId,
        gas: u64,
        value: T::Balance,
        input_data: &[u8],
    ) -> Result<U, CallError> {
        let callee = &(callee.encode())[..];
        let value = &(value.encode())[..];
        TEST_ENV_DATA.with(|test_env| {
            Decode::decode(
                &mut &(test_env.borrow_mut().call(callee, gas, value, input_data))[..],
            )
            .map_err(|_| CallError)
        })
    }

    fn create(
        code_hash: T::Hash,
        gas_limit: u64,
        value: T::Balance,
        input_data: &[u8],
    ) -> Result<T::AccountId, CreateError> {
        let code_hash = &(code_hash.encode())[..];
        let value = &(value.encode())[..];
        TEST_ENV_DATA.with(|test_env| {
            Decode::decode(
                &mut &(test_env
                    .borrow_mut()
                    .create(code_hash, gas_limit, value, input_data))[..],
            )
            .map_err(|_| CreateError)
        })
    }
}

pub enum TestEnvStorage {}

impl EnvStorage for TestEnvStorage {
    unsafe fn store(key: Key, value: &[u8]) {
        TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().store(key, value))
    }

    unsafe fn clear(key: Key) {
        TEST_ENV_DATA.with(|test_env| test_env.borrow_mut().clear(key))
    }

    unsafe fn load(key: Key) -> Option<Vec<u8>> {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().load(key))
    }
}

impl TestEnvStorage {
    /// Returns the total number of reads from the storage.
    pub fn total_reads() -> u64 {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().total_reads())
    }

    /// Returns the total number of writes to the storage.
    pub fn total_writes() -> u64 {
        TEST_ENV_DATA.with(|test_env| test_env.borrow().total_writes())
    }
}
