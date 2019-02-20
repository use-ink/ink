// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

#![no_std]

#[cfg(all(test, feature = "test-env"))]
mod tests;

use pdsl_core::{
	env::srml::Address,
	env::{Env, ContractEnv},
	memory::{
		string::String,
	},
	storage::{
		self,
		Key,
		Flush,
		alloc::{
			BumpAlloc,
			AllocateUsing,
			Initialize,
		},
	},
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
	/// Owner address.
	owner: Address,
	/// The peeps.
	peeps: storage::Vec<String>,
	/// The follows.
	following: storage::Vec<String>,
}

impl AllocateUsing for UserData {
	unsafe fn allocate_using<A>(alloc: &mut A) -> Self
	where
		A: storage::Allocator
	{
		Self {
			owner: Address::from(&[0x0; 32][..]),
			peeps: storage::Vec::allocate_using(alloc),
			following: storage::Vec::allocate_using(alloc),
		}
	}
}

impl Initialize for UserData {
	type Args = Address;

	fn initialize(&mut self, address: Self::Args) {
		self.owner = address;
	}
}

impl Flush for UserData {
	fn flush(&mut self) {
		self.peeps.flush();
		self.following.flush();
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

impl AllocateUsing for Subpeep {
	unsafe fn allocate_using<A>(alloc: &mut A) -> Self
	where
		A: pdsl_core::storage::Allocator
	{
		Self {
			peeps: storage::Vec::allocate_using(alloc),
			users: storage::HashMap::allocate_using(alloc),
			alloc: storage::alloc::CellChunkAlloc::allocate_using(alloc),
		}
	}
}

impl Initialize for Subpeep {
	type Args = ();

	fn initialize(&mut self, _args: Self::Args) {
		self.peeps.initialize(());
		self.users.initialize(());
		self.alloc.initialize(());
	}
}

impl Flush for Subpeep {
	fn flush(&mut self) {
		self.peeps.flush();
		self.users.flush();
		self.alloc.flush();
	}
}

impl Subpeep {
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
			let user_data =
				unsafe {
					UserData::allocate_using(&mut self.alloc)
				}.initialize_into(ContractEnv::caller());
			self.users.insert(username.into(), user_data);
			return true
		}
		false
	}

	/// Post a message by a user.
	pub fn peep_message(&mut self, username: &str, message: &str) {
		// Check if the caller is registered as the peeping user.
		assert_eq!(
			self
				.users
				.get(username)
				.map(|data| data.owner)
				.unwrap(),
			ContractEnv::caller()
		);
		self.peep_global(username, message);
		self
			.users
			.mutate_with(username, |user| {
				user.peeps.push(message.into())
			});
	}

	/// Make a user follow the other.
	pub fn follow(&mut self, following: &str, followed: &str) {
		// Check if the caller is registered as the following user.
		assert_eq!(
			self
				.users
				.get(following)
				.map(|data| data.owner)
				.unwrap(),
			ContractEnv::caller()
		);
		self
			.users
			.mutate_with(following, |following| {
				following.following.push(followed.into())
			});
	}
}

/// Subpeep API.
#[derive(Encode, Decode)]
enum Action {
	/// Register a new user.
	Register{username: String},
	/// Post a message by a user.
	PeepMessage{username: String, message: String},
	/// Make a user follow the other.
	Follow{following: String, followed: String},
}

fn instantiate() -> Subpeep {
	unsafe {
		let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
		AllocateUsing::allocate_using(&mut alloc)
	}
}

#[no_mangle]
pub extern "C" fn deploy() {
	instantiate().initialize_into(()).flush()
}

#[no_mangle]
pub extern "C" fn call() {
	let input = ContractEnv::input();
	let action = Action::decode(&mut &input[..]).unwrap();
	let mut subpeep = instantiate();

	match action {
		Action::Register{username} => {
			subpeep.register(&username);
		}
		Action::PeepMessage{username, message} => {
			subpeep.peep_message(&username, &message)
		}
		Action::Follow{following, followed} => {
			subpeep.follow(&following, &followed)
		}
	}
}
