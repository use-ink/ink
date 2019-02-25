use crate::storage::Key;

/// Types implementing this trait can allocate storage.
///
/// # Note
///
/// Since the current Wasm implementation is 32-bit we are
/// fine to only support allocation sizes of max 32-bit in
/// contract storage. However, for static allocator like
/// `BumpAllocator` that is meant to allocate also other
/// allocators we might need relaxed allocation sizes.
pub trait Allocate {
	/// Allocates a storage area.
	///
	/// The returned key denotes a storage region that fits for at
	/// least the given number of cells.
	fn alloc(&mut self, size: u64) -> Key;
}

/// Types implementing this trait are storage allocators.
pub trait Allocator: Allocate {
	/// Deallocates a storage area.
	///
	/// The given storage region must have been allocated by this
	/// allocator before.
	fn dealloc(&mut self, key: Key);
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
	/// Unsafe because the storage contents of the resulting instance
	/// are uninitialized. Operating on uninitialized storage may result
	/// in panics or even in undefined behaviour.
	unsafe fn allocate_using<A>(alloc: &mut A) -> Self
	where
		A: Allocate;
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
pub trait Initialize
where
	Self: Sized
{
	/// Arguments used for deployment.
	///
	/// # Note
	///
	/// - This will probably most often be `()`.
	/// - For multiple arguments use tuples.
	type Args;

	/// Initializes storage of `self` so that it can be safely accessed.
	fn initialize(&mut self, args: Self::Args);

	/// Initializes storage of `self` so that it can be safely accessed.
	fn initialize_into(self, args: Self::Args) -> Self {
		let mut this = self;
		this.initialize(args);
		this
	}
}
