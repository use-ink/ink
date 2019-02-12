
/// Provides a safe interface to an environment given a contract state.
pub struct ExecutionEnv<S>{
	/// The contract state.
	pub state: S,
}
