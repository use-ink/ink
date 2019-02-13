use pdsl_core::{
	env::{
		Env,
		ContractEnv,
		srml::{
			Address,
		},
	},
};

/// Provides a safe interface to an environment given a contract state.
pub struct ExecutionEnv<State>{
	/// The contract state.
	pub state: State,
}

impl<State> ExecutionEnv<State> {
	pub fn caller(&self) -> Address {
		ContractEnv::caller()
	}
}
