use pdsl_core::{
	storage::{
		Key,
		alloc::{
			ForwardAlloc,
		},
	},
	env::{
		Env,
		ContractEnv,
	},
};

use parity_codec::{Encode, Decode};
use lazy_static::lazy_static;
use spin::Mutex;

use alloc::string::String;

pub type Address = [u8; 32];

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
}

/// Subpeep API.
#[derive(Encode, Decode)]
enum Action {
	/// Register a new user.
	Register{username: String},
	/// Post a message by a user.
	PeepMessage{username: String, message: String},
	/// Make a user follow the other.
	Follow{following: String, followed: String},
}

#[no_mangle]
pub extern "C" fn deploy() {}

#[no_mangle]
pub extern "C" fn call() {
	use parity_codec::{Decode};
	use pdsl_core::{
		env::{Env, ContractEnv},
	};

	let input = ContractEnv::input();
	let action = Action::decode(&mut &input[..]).unwrap();

	let mut subpeep = crate::Subpeep::default();
	match action {
		Action::Register{username} => {
			subpeep.register(&username);
		}
		Action::PeepMessage{username, message} => {
			subpeep.peep_message(&username, &message)
		}
		Action::Follow{following, followed} => {
			subpeep.follow(&following, &followed)
		}
	}
}
