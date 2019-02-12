use crate::{
	contract::ContractDecl,
	state::ContractState,
	msg::Message,
};
use pdsl_core::{
	storage::{self, alloc::Initialize},
};

state! {
	/// A simple contract having just one value that can be incremented and returned.
	struct State {
		/// The simple value on the contract storage.
		val: storage::Value<u32>
	}
}

impl Initialize for State {
	type Args = ();

	fn initialize(&mut self, _args: Self::Args) {
		self.val.set(0)
	}
}

messages! {
	/// Increases the storage value by the given amount.
	Inc(by: u32);
	/// Returns the storage value.
	Get() -> u32;
}

fn instantiate() {
	ContractDecl::new("Adder")
		.using_state::<State>()
		.on_msg_mut::<Inc>(|env, by| {
			env.state.val += by
		})
		.on_msg::<Get>(|env, _| {
			*env.state.val.get()
		});
}

#[no_mangle]
fn deploy() {
	instantiate().deploy()
}

#[no_mangle]
fn call() {
	instantiate().run()
}
