//! Setup contract storage underlying an entity.

/// Types implementing this trait are initializable on the contract storage.
///
/// # Note
///
/// Some types require special initialization routines on the contract storage
/// upon creation to properly operate on it.
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
