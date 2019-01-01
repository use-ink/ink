use pdsl_core::{
	storage::{
		Key,
		alloc::{
			Allocator,
			ForwardAlloc,
		},
	},
	env::{
		Env,
		ContractEnv,
	},
};

use lazy_static::lazy_static;
use std::sync::Mutex;

pub type Address = [u8; 32];

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
	pub(crate) static ref STORAGE_ALLOC: Mutex<ForwardAlloc> = {
		Mutex::new(unsafe {
			crate::utils::ForwardAlloc::from_raw_parts(
				Key([0x0; 32])
			)
		})
	};
	pub(crate) static ref TWEETS_KEY: Key = {
		alloc(1)
	};
	pub(crate) static ref USERS_KEY: Key = {
		let ret = alloc(1);
		alloc(u32::max_value());
		ret
	};
}
