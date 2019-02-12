use crate::{
	contract::ContractDecl,
	state::ContractState,
	msg::Message,
};
use pdsl_core::{
	storage::{self, alloc::Initialize},
};

state! {
	struct State {
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
	Inc(by: u32);
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
