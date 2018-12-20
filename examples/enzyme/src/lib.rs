#![feature(alloc)]

#[cfg(all(test, feature = "test-env"))]
mod tests;

pub mod utils;

use crate::utils::*;

use parity_codec_derive::{Encode, Decode};

use pdsl_core::{
	storage::{self, Key},
	Setup,
};

use lazy_static::lazy_static;

/// A tweet done by a registered user.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Encode, Decode)]
pub struct Tweet {
	/// By whom the tweet was done.
	by: Username,
	/// The message of the tweet.
	message: String,
}

impl Tweet {
	/// Creates a new tweet from `by` with content `message`.
	pub fn new(by: String, message: String) -> Self {
		Self{by, message}
	}
}

// / The name of a registered user.
// #[derive(Debug, Clone, PartialEq, Eq)]
// #[derive(Encode, Decode)]
// pub struct Username(pub String);

/// The name of a registered user.
pub type Username = String;

/// The data of a registered user.
#[derive(Encode, Decode)]
pub struct UserData {
	/// The tweets.
	tweets: storage::Vec<String>,
	/// The follows.
	following: storage::Vec<Username>,
}

impl UserData {
	pub fn new() -> Self {
		Self{
			tweets: unsafe {
				storage::Vec::new_unchecked({
					let key = alloc(1);
					alloc(u32::max_value());
					key
				})
			},
			following: unsafe {
				storage::Vec::new_unchecked({
					let key = alloc(1);
					alloc(u32::max_value());
					key
				})
			},
		}
	}
}

/// The entire enzyme contract.
pub struct Enzyme {
	/// All tweets done by all users.
	tweets: storage::Vec<Tweet>,
	/// Database of all registered users and their data.
	users: storage::HashMap<Username, UserData>,
}

impl Default for Enzyme {
	fn default() -> Self {
		unsafe {
			Enzyme{
				tweets: storage::Vec::new_unchecked(*TWEETS_KEY),
				users: storage::HashMap::new_unchecked(*USERS_KEY),
			}
		}
	}
}

impl Setup for Enzyme {
	/// Sets up enzyme.
	///
	/// This should be called only once upon deployment of a contract.
	fn setup(&mut self) {
		// Nothing to do here at the moment.
	}
}

impl Enzyme {
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
	fn tweet_global(&mut self, username: &str, message: String) {
		self.tweets.push(Tweet::new(username.into(), message))
	}

	/// Register a new user.
	///
	/// Returns `true` if registration was successful.
	pub fn register(&mut self, username: String) -> bool {
		if self.users.get(&username).is_none() {
			self.users.insert(username, UserData::new());
			return true
		}
		false
	}

	/// Post a message by a user.
	pub fn tweet_message(&mut self, username: String, message: String) {
		self.tweet_global(&username, message.clone());
		self
			.users
			.mutate_with(&username, |user| {
				user.tweets.push(message)
			});
	}

	/// Make a user follow the other.
	pub fn follow(&mut self, following: String, followed: String) {
		self
			.users
			.mutate_with(&following, |following| {
				following.following.push(followed)
			});
	}
}

lazy_static! {
	static ref TWEETS_KEY: Key = {
		alloc(1)
	};
	static ref USERS_KEY: Key = {
		let ret = alloc(1);
		alloc(u32::max_value());
		ret
	};
}

/// Enzyme API.
#[derive(Encode, Decode)]
enum Action {
	/// Register a new user.
	Register{username: String},
	/// Post a message by a user.
	TweetMessage{username: String, message: String},
	/// Make a user follow the other.
	Follow{following: String, followed: String},
}

#[no_mangle]
pub extern "C" fn deploy() {
	Enzyme::default().setup()
}

#[no_mangle]
pub extern "C" fn call() {
	use parity_codec::{Decode};
	use pdsl_core::{
		env::{Env, ContractEnv},
	};

	let input = ContractEnv::input();
	let action = Action::decode(&mut &input[..]).unwrap();

	let mut enzyme = Enzyme::default();
	match action {
		Action::Register{username} => {
			enzyme.register(username);
		}
		Action::TweetMessage{username, message} => {
			enzyme.tweet_message(username, message)
		}
		Action::Follow{following, followed} => {
			enzyme.follow(following, followed)
		}
	}
}
