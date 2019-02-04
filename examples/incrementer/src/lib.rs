#![cfg_attr(not(all(test, feature = "test-env")), no_std)]

use pdsl_core::{
	env::{Env, ContractEnv},
	storage::{
		alloc,
		Key,
		Value,
		alloc::{
			AllocateUsing,
			Initialize,
			MaybeUninitialized,
		},
	},
};

/// An incrementer smart contract.
///
/// Can only increment and return its current value.
struct Incrementer {
	/// The current value stored in the storage.
	current: Value<u32>,
}

impl Incrementer {
	/// Increments the current value.
	pub fn inc(&mut self, by: u32) {
		self.current += by;
	}

	/// Returns the current value.
	pub fn get(&self) -> u32 {
		*self.current
	}
}

impl AllocateUsing for Incrementer {
	unsafe fn allocate_using<A>(alloc: &mut A) -> MaybeUninitialized<Self>
	where
		A: pdsl_core::storage::Allocator,
	{
		Self {
			current: AllocateUsing::allocate_using(alloc).unwrap(),
		}.into()
	}
}

impl Initialize for Incrementer {
	type Args = ();

	fn initialize(&mut self, _args: Self::Args) {
		self.current.set(0);
	}
}

#[derive(parity_codec::Encode, parity_codec::Decode)]
enum Action {
	Inc(u32),
	Get,
}

/// The allocation key for the bump allocator.
const ALLOC_KEY: Key = Key([0x0; 32]);

fn ret<T>(val: T) -> !
where
	T: parity_codec::Encode,
{
	ContractEnv::return_(&val.encode())
}

#[no_mangle]
pub extern "C" fn deploy() {
	unsafe {
		let mut alloc = alloc::BumpAlloc::from_raw_parts(ALLOC_KEY);
		Incrementer::allocate_using(&mut alloc).initialize(());
	}
}

#[no_mangle]
pub extern "C" fn call() {
	use parity_codec::{Decode};

	let input = ContractEnv::input();
	let action = Action::decode(&mut &input[..]).unwrap();
	let mut incrementer = unsafe {
		let mut alloc = alloc::BumpAlloc::from_raw_parts(ALLOC_KEY);
		Incrementer::allocate_using(&mut alloc).unwrap()
	};

	match action {
		Action::Inc(by) => {
			incrementer.inc(by)
		}
		Action::Get => {
			ret(&incrementer.get())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get() {
		let mut incrementer = unsafe {
			let mut alloc = alloc::BumpAlloc::from_raw_parts(ALLOC_KEY);
			Incrementer::allocate_using(&mut alloc).initialize(())
		};
		assert_eq!(incrementer.get(), 0)
	}

	#[test]
	fn test_set() {
		let mut incrementer = unsafe {
			let mut alloc = alloc::BumpAlloc::from_raw_parts(ALLOC_KEY);
			Incrementer::allocate_using(&mut alloc).initialize(())
		};
		incrementer.inc(1);
		assert_eq!(incrementer.get(), 1);
		incrementer.inc(42);
		assert_eq!(incrementer.get(), 43);
	}
}
