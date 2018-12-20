use pdsl_core::storage::Key;
use pdsl_core::env::{Env, ContractEnv};

use lazy_static::lazy_static;
use std::sync::Mutex;

pub type Address = [u8; 32];

pub struct StorageAlloc {
	/// Current key with highest offset.
	cur: Key,
}

impl Default for StorageAlloc {
	fn default() -> Self {
		Self{
			cur: Key([0; 32])
		}
	}
}

impl StorageAlloc {
	pub fn alloc(&mut self, size: u32) -> Key {
		let ret = self.cur;
		self.cur = Key::with_offset(ret, size);
		ret
	}
}

/// Allocates an amount of contract storage.
///
/// Doesn't actually touch the storage but keeps it from
/// getting reused by another instance.
pub fn alloc(size: u32) -> Key {
	STORAGE_ALLOC.lock().unwrap().alloc(size)
}

/// Return current contract caller as 32-bytes array.
pub fn caller_as_array() -> [u8; 32] {
	let caller_as_vec = ContractEnv::caller();
	assert_eq!(caller_as_vec.len(), 32);
	let mut buffer: [u8; 32] = [0; 32];
	buffer.copy_from_slice(&caller_as_vec);
	buffer
}

lazy_static! {
	static ref STORAGE_ALLOC: Mutex<StorageAlloc> = {
		Mutex::new(crate::utils::StorageAlloc::default())
	};
}
