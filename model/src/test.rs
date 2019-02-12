use crate::{
	contract::ContractDecl,
	state::ContractState,
	msg::Message,
};
use pdsl_core::{
	storage::{self, alloc::Initialize},
};
use crate as pdsl_model; // TODO: We really don't want this as a hack to make state! work.

state! {
	/// A simple contract having just one value that can be incremented and returned.
	struct Adder {
		/// The simple value on the contract storage.
		val: storage::Value<u32>
	}
}

messages! {
	/// Increases the storage value by the given amount.
	Inc(by: u32);
	/// Returns the storage value.
	Get() -> u32;
}

fn instantiate() {
	ContractDecl::new::<Adder>()
		.on_deploy(|env, init_val| {
			env.state.val.set(init_val)
		})
		.on_msg_mut::<Inc>(|env, by| {
			env.state.val += by
		})
		.on_msg::<Get>(|env, _| {
			*env.state.val.get()
		});
}

#[no_mangle]
fn deploy() {
	// instantiate().deploy()
}

#[no_mangle]
fn call() {
	// instantiate().run()
}
