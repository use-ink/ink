//! Setup contract storage underlying an entity.

/// Types implementing this trait are deployable on the chain.
pub trait Setup {
	/// Setup contract storage underlying to `self`.
	///
	/// This initializes contract storage used by the entity
	/// to whatever state it expects to be for operating on it.
	///
	/// # Note
	///
	/// This should be executed only once per instance.
	fn setup(&mut self);
}
