#![feature(proc_macro_hygiene)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]
#![feature(alloc)]

use parity_codec_derive;

pub mod incrementer {
	use pdsl_core::env::{Env, ContractEnv};
	use pdsl_core::storage::{Key, Stored};

	#[derive(parity_codec_derive::Encode, parity_codec_derive::Decode)]
	enum Action {
		Inc(u32),
		Get,
	}

	const COUNTER_KEY: Key = Key([1; 32]);

	#[no_mangle]
	pub extern "C" fn call() {
		use parity_codec::{Decode, Encode};

		let input = ContractEnv::input();
		let action = Action::decode(&mut &input[..]).unwrap();
		let mut storage = Stored::<u32>::from(COUNTER_KEY);

		match action {
			Action::Inc(by) => {
				let mut counter = storage.try_load().unwrap_or(0);
				counter += by;
				storage.store(&counter);
			}
			Action::Get => {
				let raw_counter = COUNTER_KEY.load().unwrap_or(vec![]);
				ContractEnv::return_(&raw_counter);
			}
		}
	}
}
