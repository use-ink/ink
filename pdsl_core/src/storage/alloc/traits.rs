use crate::storage::Key;

/// Types implementing this trait are storage allocators.
pub trait Allocator {
	/// Allocates a storage area.
	///
	/// The returned key denotes a storage region that fits for at
	/// least the given number of cells.
	fn alloc(&mut self, size: u32) -> Key;

	/// Deallocates a storage area.
	///
	/// The given storage region must have been allocated by this
	/// allocator before.
	fn dealloc(&mut self, key: Key);
}

/// Returned upon storage allocation.
///
/// Holds the actual allocated entity and provides a somewhat safer interface to accessing it.
pub struct MaybeUninitialized<T>(T);

impl<T> MaybeUninitialized<T> {
	/// Unwraps the inner value avoiding all safety checks.
	///
	/// # Note
	///
	/// Use this if the allocated entity is known to have already been initialized,
	/// for example during deployment of a smart contract.
	pub fn unwrap(self) -> T {
		self.0
	}
}

impl<A, T> MaybeUninitialized<T>
where
	T: Initialize<Args = A>,
{
	/// Initialize the inner entity with the given arguments and returns the result.
	pub fn initialize(self, args: A) -> T {
		let mut inner = self.unwrap();
		inner.initialize(args);
		inner
	}
}

impl<T> From<T> for MaybeUninitialized<T> {
	fn from(value: T) -> Self {
		MaybeUninitialized(value)
	}
}

/// Types implementing this trait can be allocated on the storage by storage allocators.
pub trait AllocateUsing
where
	Self: Sized
{
	/// Allocates an uninitialized instance of `Self` using
	/// the given storage allocator.
	///
	/// # Safety
	///
	/// Instances created this way might have uninitialized storage.
	/// Accessing those instances will panic the execution or might
	/// result in other unintended behaviour.
	unsafe fn allocate_using<A>(alloc: &mut A) -> MaybeUninitialized<Self>
	where
		A: Allocator;
}

/// Types implementing this trait require initialization of their storage contents
/// after allocation before they can be used.
///
/// # Example
///
/// For example types like [`Value`](struct.Value.html) have uninitialized
/// associated storage. Accessing a newly allocated instance of [`Value`](struct.Value.html)
/// would result in a panic or even undefined behaviour.
/// To circumvent this it is required to initialize its associated contract storage
/// via [`initialize`](trait.Initialize.html#method.initialize).
pub trait Initialize {
	/// Arguments used for deployment.
	///
	/// # Note
	///
	/// - This will probably most often be `()`.
	/// - For multiple arguments use tuples.
	type Args;

	/// Initializes storage of `self` so that it can be safely accessed.
	fn initialize(&mut self, args: Self::Args);
}
