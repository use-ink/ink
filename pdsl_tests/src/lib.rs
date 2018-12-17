#![feature(alloc_error_handler)]
#![feature(alloc)]

pub mod incrementer {
	use pdsl_core::env::{Env, ContractEnv};
	use pdsl_core::storage::{Key, cell::SyncCell};

	#[derive(parity_codec_derive::Encode, parity_codec_derive::Decode)]
	enum Action {
		Inc(u32),
		Get,
	}

	const COUNTER_KEY: Key = Key([1; 32]);

	#[no_mangle]
	pub extern "C" fn deploy() {
		unsafe {
			SyncCell::new_unchecked(COUNTER_KEY).set(0)
		}
	}

	#[no_mangle]
	pub extern "C" fn call() {
		use parity_codec::{Decode, Encode};

		let input = ContractEnv::input();
		let action = Action::decode(&mut &input[..]).unwrap();

		let mut counter = unsafe {
			SyncCell::<u32>::new_unchecked(COUNTER_KEY)
		};

		match action {
			Action::Inc(by) => {
				counter.mutate_with(|counter| *counter += by);
			}
			Action::Get => {
				ContractEnv::return_(
					&counter.get().unwrap_or(&0).encode()
				);
			}
		}
	}
}
