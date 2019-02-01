#![no_std]

pub mod incrementer {
	use pdsl_core::{
		env::{Env, ContractEnv},
		storage::{
			alloc,
			Key,
			cell::SyncCell,
		},
	};

	#[derive(parity_codec::Encode, parity_codec::Decode)]
	enum Action {
		Inc(u32),
		Get,
	}

	const ALLOC_KEY: Key = Key([0x0; 32]);

	#[no_mangle]
	pub extern "C" fn deploy() {
		unsafe {
			let mut alloc = alloc::BumpAlloc::from_raw_parts(ALLOC_KEY);
			SyncCell::new_using_alloc(&mut alloc).set(0)
		}
	}

	#[no_mangle]
	pub extern "C" fn call() {
		use parity_codec::{Decode, Encode};

		let input = ContractEnv::input();
		let action = Action::decode(&mut &input[..]).unwrap();

		let mut counter = unsafe {
			let mut alloc = alloc::BumpAlloc::from_raw_parts(ALLOC_KEY);
			SyncCell::new_using_alloc(&mut alloc)
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
