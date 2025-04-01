// Copyright (C) Use Ink (UK) Ltd.
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

use crate::types::Balance;
use ink_primitives::{
    AccountId,
    Address,
    H256,
    U256,
};
use scale::KeyedVec;
use std::collections::HashMap;

const BALANCE_OF: &[u8] = b"balance:";
const STORAGE_OF: &[u8] = b"contract-storage:";
const CONTRACT_PREFIX: &[u8] = b"contract:";
const MSG_HANDLER_OF: &[u8] = b"message-handler:";
const CODE_HASH_OF: &[u8] = b"code-hash:";

/// Returns the database key under which to find the balance for contract `who`.
pub fn balance_of_key(who: &Address) -> [u8; 32] {
    let keyed = who.0.to_vec().to_keyed_vec(BALANCE_OF);
    let mut hashed_key: [u8; 32] = [0; 32];
    super::hashing::blake2b_256(&keyed[..], &mut hashed_key);
    hashed_key
}

/// Returns the database key under which to find the storage for contract `who`.
pub fn storage_of_contract_key(who: &Address, key: &[u8]) -> [u8; 32] {
    let keyed = who
        .as_bytes()
        .to_vec()
        .to_keyed_vec(key)
        .to_keyed_vec(STORAGE_OF);
    let mut hashed_key: [u8; 32] = [0; 32];
    super::hashing::blake2b_256(&keyed[..], &mut hashed_key);
    hashed_key
}

pub type MessageHandler = fn(Vec<u8>) -> Vec<u8>;

pub fn contract_key(f: MessageHandler) -> [u8; 32] {
    let f = f as usize;
    let f = f.to_le_bytes();
    let keyed = f.to_vec().to_keyed_vec(CONTRACT_PREFIX);
    let mut ret: [u8; 32] = [0; 32];
    super::hashing::blake2b_256(&keyed[..], &mut ret);
    ret
}

pub fn message_handler_of_contract_key(key: &[u8]) -> [u8; 32] {
    let keyed = key.to_vec().to_keyed_vec(MSG_HANDLER_OF);
    let mut hashed_key: [u8; 32] = [0; 32];
    super::hashing::blake2b_256(&keyed[..], &mut hashed_key);
    hashed_key
}

pub fn code_hash_for_addr(addr: &Address) -> [u8; 32] {
    let key = addr.0;
    let keyed = key.to_keyed_vec(CODE_HASH_OF);
    let mut hashed_key: [u8; 32] = [0; 32];
    super::hashing::blake2b_256(&keyed[..], &mut hashed_key);
    hashed_key
}

/// The chain database.
///
/// Everything is stored in here: contracts, balances, contract storage, etc.
/// Just like in Substrate a prefix hash is computed for every contract.
#[derive(Default)]
pub struct Database {
    hmap: HashMap<Vec<u8>, Vec<u8>>,
    fmap: HashMap<Vec<u8>, MessageHandler>,
}

impl Database {
    /// Creates a new database instance.
    pub fn new() -> Self {
        Database {
            hmap: HashMap::new(),
            fmap: HashMap::new(),
        }
    }

    /// Returns the amount of entries in the database.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.hmap.len()
    }

    /// Returns a reference to the value corresponding to the key.
    fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.hmap.get(key)
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get_from_contract_storage(
        &self,
        addr: &Address,
        key: &[u8],
    ) -> Option<&Vec<u8>> {
        let hashed_key = storage_of_contract_key(addr, key);
        self.hmap.get(hashed_key.as_slice())
    }

    /// Inserts `value` into the contract storage of `addr` at storage key `key`.
    pub fn insert_into_contract_storage(
        &mut self,
        addr: &Address,
        key: &[u8],
        value: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let hashed_key = storage_of_contract_key(addr, key);
        self.hmap.insert(hashed_key.to_vec(), value)
    }

    /// Removes the value at the contract storage of `addr` at storage key `key`.
    pub fn remove_contract_storage(
        &mut self,
        addr: &Address,
        key: &[u8],
    ) -> Option<Vec<u8>> {
        let hashed_key = storage_of_contract_key(addr, key);
        self.hmap.remove(hashed_key.as_slice())
    }

    /// Removes a key from the storage, returning the value at the key if the key
    /// was previously in storage.
    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.hmap.remove(key)
    }

    /// Sets the value of the entry, and returns the entry's old value.
    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<Vec<u8>> {
        self.hmap.insert(key, value)
    }

    /// Clears the database, removing all key-value pairs.
    pub fn clear(&mut self) {
        self.hmap.clear();
    }

    /// Returns the balance of the contract at `addr`, if available.
    pub fn get_acc_balance(&self, _addr: &AccountId) -> Option<Balance> {
        todo!()
    }

    /// Sets the balance of `addr` to `new_balance`.
    pub fn set_acc_balance(&mut self, _addr: &AccountId, _new_balance: Balance) {
        todo!()
    }

    pub fn get_balance(&self, addr: &Address) -> Option<U256> {
        let hashed_key = balance_of_key(addr);
        self.get(&hashed_key).map(|encoded_balance| {
            scale::Decode::decode(&mut &encoded_balance[..])
                .expect("unable to decode balance from database")
        })
    }

    /// Sets the balance of `addr` to `new_balance`.
    pub fn set_balance(&mut self, addr: &Address, new_balance: U256) {
        let hashed_key = balance_of_key(addr);
        let encoded_balance = scale::Encode::encode(&new_balance);
        self.hmap
            .entry(hashed_key.to_vec())
            .and_modify(|v| *v = encoded_balance.clone())
            .or_insert(encoded_balance);
    }

    pub fn set_contract_message_handler(&mut self, handler: MessageHandler) -> [u8; 32] {
        let key = contract_key(handler);
        let hashed_key = message_handler_of_contract_key(&key);
        self.fmap
            .entry(hashed_key.to_vec())
            .and_modify(|x| *x = handler)
            .or_insert(handler);
        key
    }

    /// Returns the message handler for a code hash.
    pub fn get_contract_message_handler(&mut self, code_hash: &H256) -> MessageHandler {
        let hashed_key = message_handler_of_contract_key(&code_hash.0);
        *self.fmap.get(hashed_key.as_slice()).unwrap()
    }

    pub fn set_code_hash(&mut self, addr: &Address, code_hash: &H256) {
        let hashed_key = code_hash_for_addr(addr);
        self.hmap
            .entry(hashed_key.to_vec())
            .and_modify(|x| *x = code_hash.as_bytes().to_vec())
            .or_insert(code_hash.as_bytes().to_vec());
    }

    pub fn get_code_hash(&self, addr: &Address) -> Option<H256> {
        let hashed_key = code_hash_for_addr(addr);
        self.get(&hashed_key)
            .cloned()
            .map(|v| H256::from_slice(v.as_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Address,
        Database,
    };

    #[test]
    fn basic_operations() {
        let mut database = Database::new();
        let key1 = vec![42];
        let key2 = vec![43];
        let val1 = vec![44];
        let val2 = vec![45];
        let val3 = vec![46];

        assert_eq!(database.len(), 0);
        assert_eq!(database.get(&key1), None);
        assert_eq!(database.insert(key1.clone(), val1.clone()), None);
        assert_eq!(database.get(&key1), Some(&val1));
        assert_eq!(database.insert(key1.clone(), val2.clone()), Some(val1));
        assert_eq!(database.get(&key1), Some(&val2));
        assert_eq!(database.insert(key2.clone(), val3.clone()), None);
        assert_eq!(database.len(), 2);
        assert_eq!(database.remove(&key2), Some(val3));
        assert_eq!(database.len(), 1);
        database.clear();
        assert_eq!(database.len(), 0);
    }

    #[test]
    fn contract_storage() {
        let addr = Address::from([1; 20]);
        let mut storage = Database::new();
        let key1 = vec![42];
        let key2 = vec![43];
        let val1 = vec![44];
        let val2 = vec![45];
        let val3 = vec![46];

        assert_eq!(storage.len(), 0);
        assert_eq!(storage.get_from_contract_storage(&addr, &key1), None);
        assert_eq!(
            storage.insert_into_contract_storage(&addr, &key1, val1.clone()),
            None
        );
        assert_eq!(storage.get_from_contract_storage(&addr, &key1), Some(&val1));
        assert_eq!(
            storage.insert_into_contract_storage(&addr, &key1, val2.clone()),
            Some(val1)
        );
        assert_eq!(storage.get_from_contract_storage(&addr, &key1), Some(&val2));
        assert_eq!(
            storage.insert_into_contract_storage(&addr, &key2, val3.clone()),
            None
        );
        assert_eq!(storage.len(), 2);
        assert_eq!(storage.remove_contract_storage(&addr, &key2), Some(val3));
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.remove_contract_storage(&addr, &key1), Some(val2));
        assert_eq!(storage.len(), 0);
    }
}
