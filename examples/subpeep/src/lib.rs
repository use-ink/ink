#![feature(alloc)]

#[cfg(all(test, feature = "test-env"))]
mod tests;

pub mod utils;

use pdsl_core::storage;
use parity_codec_derive::{Encode, Decode};

/// A tweet done by a registered user.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Encode, Decode)]
pub struct Tweet {
	/// By whom the tweet was done.
	by: String,
	/// The message of the tweet.
	message: String,
}

impl Tweet {
	/// Creates a new tweet from `by` with content `message`.
	pub fn new(by: String, message: String) -> Self {
		Self{by, message}
	}
}

/// The data of a registered user.
#[derive(Encode, Decode)]
pub struct UserData {
	/// The tweets.
	tweets: storage::Vec<String>,
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
			tweets: storage::Vec::new_using_alloc(alloc),
			following: storage::Vec::new_using_alloc(alloc),
		}
	}
}

/// The entire subpeep contract.
pub struct Subpeep {
	/// All tweets done by all users.
	tweets: storage::Vec<Tweet>,
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
			tweets: storage::Vec::new_using_alloc(alloc),
			users: storage::HashMap::new_using_alloc(alloc),
			alloc: storage::alloc::CellChunkAlloc::new_using_alloc(alloc),
		}
	}

	/// Returns all recent global posts as vector.
	pub(crate) fn recent_tweets(&self, amount: usize) -> Vec<Tweet> {
		self
			.tweets
			.iter()
			.rev()
			.take(amount)
			.cloned()
			.collect()
	}

	/// Returns the `n` most recent tweets of the given user.
	///
	/// Returns `None` if the user does not exist.
	pub(crate) fn recent_user_tweets(
		&self,
		amount: usize,
		username: &str,
	) -> Option<Vec<Tweet>> {
		self
			.users
			.get(username)
			.map(|user| {
				user
					.tweets
					.iter()
					.rev()
					.take(amount)
					.cloned()
					.map(|message| {
						Tweet::new(username.into(), message)
					})
					.collect()
			})
	}

	/// Posts a message to the global channel.
	/// 
	/// Will only ever store the latest 10 messages in the channel at most.
	fn tweet_global(&mut self, username: &str, message: &str) {
		self.tweets.push(Tweet::new(username.into(), message.into()))
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
	pub fn tweet_message(&mut self, username: &str, message: &str) {
		self.tweet_global(username, message);
		self
			.users
			.mutate_with(username, |user| {
				user.tweets.push(message.into())
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
