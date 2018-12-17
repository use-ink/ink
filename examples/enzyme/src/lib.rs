#![feature(alloc)]

#[cfg(all(test, feature = "test-env"))]
mod tests;

pub type Address = [u8; 32];

use parity_codec_derive::{Encode, Decode};

use pdsl_core::{
	env::{Env, ContractEnv},
	storage::{self, Key, cell::SyncCell},
	Setup,
};

use lazy_static::lazy_static;
use std::sync::Mutex;

struct StorageAlloc {
	/// Current key with highest offset.
	cur: Key,
}

impl Default for StorageAlloc {
	fn default() -> Self {
		Self{
			cur: Key([0; 32])
		}
	}
}

impl StorageAlloc {
	pub fn alloc(&mut self, size: u32) -> Key {
		let ret = self.cur;
		self.cur = Key::with_offset(ret, size);
		ret
	}
}

/// Allocates an amount of contract storage.
///
/// Doesn't actually touch the storage but keeps it from
/// getting reused by another instance.
fn alloc(size: u32) -> Key {
	STORAGE_ALLOC.lock().unwrap().alloc(size)
}

pub type Username = String;
pub type Tweet = String;

/// Decentralized FAKE-NEWs distribution.
pub struct Enzyme {
	/// Latest global tweet ID.
	global_tweets_latest: SyncCell<u32>,
	/// The last tweets posted by all users.
	global_tweets: storage::Vec<Tweet>,
	/// The users address. (For Authentification purpose.)
	auth: storage::HashMap<Username, Address>,
	/// All tweets posted by the user.
	user_tweets: storage::HashMap<Username, Vec<Tweet>>,
	/// Who this user is following.
	following: storage::HashMap<Username, Vec<Username>>,
}

impl Default for Enzyme {
	fn default() -> Self {
		unsafe {
			Enzyme{
				global_tweets_latest: SyncCell::new_unchecked(*GLOBAL_TWEETS_LATEST_KEY),
				global_tweets: storage::Vec::new_unchecked(*GLOBAL_TWEETS_KEY),
				auth: storage::HashMap::new_unchecked(*AUTH_KEY),
				user_tweets: storage::HashMap::new_unchecked(*USER_TWEETS_KEY),
				following: storage::HashMap::new_unchecked(*FOLLOWING_KEY),
			}
		}
	}
}

/// Return current contract caller as 32-bytes array.
pub fn caller_as_array() -> [u8; 32] {
	let caller_as_vec = ContractEnv::caller();
	assert_eq!(caller_as_vec.len(), 32);
	let mut buffer: [u8; 32] = [0; 32];
	buffer.copy_from_slice(&caller_as_vec);
	buffer
}

impl Setup for Enzyme {
	/// Sets up enzyme.
	///
	/// This should be called only once upon deployment of a contract.
	fn setup(&mut self) {
		self.auth.setup();
		self.user_tweets.setup();
		self.following.setup();
		// Fill global tweets after normal setup.
		self.global_tweets_latest.set(0);
		assert_eq!(self.global_tweets.len(), 0);
		for _ in 0..10 {
			self.global_tweets.push(String::from(""));
		}
		assert_eq!(self.global_tweets.len(), 10);
	}
}

impl Enzyme {
	/// Maximum number of global posts.
	const MAX_GLOBAL_TWEETS: u32 = 10;

	/// Returns all recent global posts as vector.
	pub(crate) fn global_tweets(&self) -> Vec<Tweet> {
		println!("Enzyme::global_tweets - start");
		let mut buffer = Vec::new();
		for (i, global_tweet) in self.global_tweets.iter().take(10).enumerate() {
			println!("Enzyme::global_tweets - ({:?}, {:?})", i, global_tweet);
			buffer.push(global_tweet.clone());
		}
		// println!("Enzyme::global_tweets - 2");
		// buffer.rotate_left(*self.global_tweets_latest.get() as usize);
		// println!("Enzyme::global_tweets - 3");
		// buffer.reverse();
		println!("Enzyme::global_tweets - end");
		buffer
	}

	/// Returns the `n` most recent tweets of the given user.
	pub(crate) fn recent_user_tweets(&self, n: u32, username: Username) -> Vec<Tweet> {
		println!("Enzyme::recent_user_tweets - 0 - username = {:?}", username);
		let mut buffer = Vec::new();
		println!("Enzyme::recent_user_tweets - 1");
		if let Some(user_tweets) = self.user_tweets.get(&username) {
			println!("Enzyme::recent_user_tweets - user_tweets.len() = {:?}", user_tweets.len());
			println!("Enzyme::recent_user_tweets - user_tweets = {:?}", user_tweets);
			let len = user_tweets.len();
			if len == 0 {
				return Vec::new()
			}
			let last_index: i32 = std::cmp::max(0, len as i32);
			let start_index: i32 = std::cmp::max(0, last_index - n as i32);
			println!("Enzyme::recent_user_tweets - last_index = {:?}", last_index);
			println!("Enzyme::recent_user_tweets - start_index = {:?}", start_index);
			for i in start_index..last_index {
				buffer.push(user_tweets[i as usize].clone());
			}
		}
		println!("Enzyme::recent_user_tweets - end");
		buffer
	}

	/// Posts a message to the global channel.
	/// 
	/// Will only ever store the latest 10 messages in the channel at most.
	fn tweet_global(&mut self, message: String) {
		println!("Enzyme::tweet_global - 0");
		let latest_index = *self.global_tweets_latest.get().unwrap_or(&0);
		println!("Enzyme::tweet_global - latest_index¹ = {:?}", latest_index);
		// Reset latest_index if it reached the maximum number of global posts.
		let latest_index = if latest_index == Self::MAX_GLOBAL_TWEETS {
			self.global_tweets_latest.set(0);
			0
		} else {
			self.global_tweets_latest.set(latest_index + 1);
			latest_index
		};
		println!("Enzyme::tweet_global - latest_index² = {:?}", latest_index);
		self.global_tweets.replace(latest_index, || message);
		self.global_tweets_latest.set(latest_index + 1);
	}

	/// Register a new user.
	pub fn register(&mut self, username: String) {
		// Only register the caller for the given username
		// if there is not already a registration for it.
		if self.auth.get(&username).is_none() {
			println!("Enzyme::register - success");
			self.auth.insert(username, caller_as_array());
		} else {
			println!("Enzyme::register - fail");
		}
	}

	/// Post a message by a user.
	pub fn tweet_message(&mut self, username: String, message: String) {
		println!(
			"Enzyme::tweet_message - from = {:?}, message = {:?}",
			username,
			message,
		);
		if let Some(registered_address) = self.auth.get(&username) {
			println!("Enzyme::tweet_message - Some({:?})", registered_address);
			if registered_address == &caller_as_array() {
				println!("Enzyme::tweet_message - *is* caller");
				// There is currently no way to mutate inplace
				// while using storage map. This feature will be
				// added in the future.
				let mut tweets = self
					.user_tweets
					.get(&username)
					.unwrap_or(&Vec::new())
					.clone();
				self.tweet_global(message.clone());
				tweets.push(message);
				println!("Enzyme::tweet_message - tweets = {:?}", tweets);
				self.user_tweets.insert(username, tweets);
			} else {
				println!("Enzyme::tweet_message - *not* caller");
			}
		} else {
			println!("Enzyme::tweet_message - None")
		}
		println!("Enzyme::tweet_message - end");
	}

	/// Make a user follow the other.
	pub fn follow(&mut self, following: String, followed: String) {
		if let Some(registered_following) = self.auth.get(&following) {
			if registered_following != &caller_as_array() {
				return;
			}
			if self.auth.get(&followed).is_none() {
				return;
			}
			let mut list_followed = self
				.following
				.get(&following)
				.unwrap()
				.clone();
			if list_followed.contains(&followed) {
				list_followed.push(followed);
			}
			self.following.insert(following, list_followed);
		}
	}
}

lazy_static! {
	static ref STORAGE_ALLOC: Mutex<StorageAlloc> = {
		Mutex::new(StorageAlloc::default())
	};
	static ref GLOBAL_TWEETS_LATEST_KEY: Key = {
		alloc(1)
	};
	static ref GLOBAL_TWEETS_KEY: Key = {
		let ret = alloc(1);
		alloc(u32::max_value());
		ret
	};
	static ref AUTH_KEY: Key = {
		let ret = alloc(1);
		alloc(u32::max_value());
		ret
	};
	static ref USER_TWEETS_KEY: Key = {
		let ret = alloc(1);
		alloc(u32::max_value());
		ret
	};
	static ref FOLLOWING_KEY: Key = {
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
			enzyme.register(username)
		}
		Action::TweetMessage{username, message} => {
			enzyme.tweet_message(username, message)
		}
		Action::Follow{following, followed} => {
			enzyme.follow(following, followed)
		}
	}
}
