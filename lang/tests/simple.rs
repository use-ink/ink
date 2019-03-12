#[macro_use]
extern crate pdsl_lang;

use pdsl_core::{
	storage,
};

contract! {
	/// A simple contract that has a value that can be
	/// incremented, returned and compared.
	struct Incrementer {
		/// The internal value.
		value: storage::Value<u32>,
	}

	impl Incrementer {
        /// Automatically called when the contract is deployed.
		pub(external) fn on_deploy(&mut self, init_value: u32) {
            self.value.set(init_value);
        }

		/// Increments the internal counter.
		pub(external) fn inc(&mut self, by: u32) {
			self.value += by
		}

		/// Returns the internal counter.
		pub(external) fn get(&self) -> u32 {
			*self.value
		}

		/// Returns `true` if `x` is greater than the internal value.
		pub(external) fn compare(&self, x: u32) -> bool {
			x > *self.value
		}
	}
}
