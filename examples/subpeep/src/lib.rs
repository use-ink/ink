#![no_std]

#[cfg(all(test, feature = "test-env"))]
mod tests;
mod utils;

use pdsl_core::{
	memory::{
		string::String,
	},
	storage,
};
use parity_codec::{Encode, Decode};

/// A peep done by a registered user.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct Peep {
	/// By whom the peep was done.
	by: String,
	/// The message of the peep.
	message: String,
}

impl Peep {
	/// Creates a new peep from `by` with content `message`.
	pub fn new(by: String, message: String) -> Self {
		Self{by, message}
	}
}

/// The data of a registered user.
#[derive(Debug, Encode, Decode)]
pub struct UserData {
	/// The peeps.
	peeps: storage::Vec<String>,
	/// The follows.
	following: storage::Vec<String>,
}

impl UserData {
	/// Creates new user data using the given allocator.
	///
	/// # Note
	///
	/// The `CellChunkAlloc` should be preferred here since
	/// allocations of this type are dynamic. For this reason
	/// the `Subpeep` type has a built-in `CellChunkAlloc`.
	pub unsafe fn new_using_alloc<A>(alloc: &mut A) -> Self
	where
		A: storage::Allocator
	{
		Self {
			peeps: storage::Vec::new_using_alloc(alloc),
			following: storage::Vec::new_using_alloc(alloc),
		}
	}
}

/// The entire subpeep contract.
pub struct Subpeep {
	/// All peeps done by all users.
	peeps: storage::Vec<Peep>,
	/// Database of all registered users and their data.
	users: storage::HashMap<String, UserData>,
	/// The allocator for newly allocated entities.
	alloc: storage::alloc::CellChunkAlloc,
}

impl Default for Subpeep {
	fn default() -> Self {
		unsafe {
			Subpeep::new_using_alloc(
				&mut* utils::STORAGE_ALLOC.lock().unwrap()
			)
		}
	}
}

impl Subpeep {
	/// Creates new subpeep platform using the given allocator.
	///
	/// # Note
	///
	/// The `ForwardAlloc` should be preferred here since there is
	/// normally only one instance of this type and it can be registered
	/// during contract compiĺe-time.
	unsafe fn new_using_alloc<A>(alloc: &mut A) -> Self
	where
		A: pdsl_core::storage::Allocator
	{
		Self {
			peeps: storage::Vec::new_using_alloc(alloc),
			users: storage::HashMap::new_using_alloc(alloc),
			alloc: storage::alloc::CellChunkAlloc::new_using_alloc(alloc),
		}
	}

	/// Posts a message to the global channel.
	/// 
	/// Will only ever store the latest 10 messages in the channel at most.
	fn peep_global(&mut self, username: &str, message: &str) {
		self.peeps.push(Peep::new(username.into(), message.into()))
	}

	/// Register a new user.
	///
	/// Returns `true` if registration was successful.
	pub fn register(&mut self, username: &str) -> bool {
		if self.users.get(username).is_none() {
			let user_data = unsafe { UserData::new_using_alloc(&mut self.alloc) };
			self.users.insert(username.into(), user_data);
			return true
		}
		false
	}

	/// Post a message by a user.
	pub fn peep_message(&mut self, username: &str, message: &str) {
		self.peep_global(username, message);
		self
			.users
			.mutate_with(username, |user| {
				user.peeps.push(message.into())
			});
	}

	/// Make a user follow the other.
	pub fn follow(&mut self, following: &str, followed: &str) {
		self
			.users
			.mutate_with(following, |following| {
				following.following.push(followed.into())
			});
	}
}
