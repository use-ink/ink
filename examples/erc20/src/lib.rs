#![no_std]

use pdsl_core::{
	storage,
	env::{Env, ContractEnv, srml::Address},
};
use parity_codec::{Encode, Decode};

type Balance = u64;

/// Returns the zero address.
fn zero_address() -> Address {
	Address::from([0x0_u8; 32].as_ref())
}

/// The storage data that is hold by the ERC-20 token.
#[derive(Debug, Encode, Decode)]
pub struct Erc20Token {
	/// All peeps done by all users.
	balances: storage::HashMap<Address, Balance>,
	/// Balances that are spendable by non-owners.
	///
	/// # Note
	///
	/// Mapping: (from, to) -> allowed
	allowances: storage::HashMap<(Address, Address), Balance>,
	/// The total supply.
	total_supply: storage::Value<Balance>,
	/// The allocator for newly allocated entities.
	alloc: storage::alloc::CellChunkAlloc,
}

impl Erc20Token {
	/// Returns the total number of tokens in existence.
	pub fn total_supply(&self) -> Balance {
		*self.total_supply
	}

	/// Returns the balance of the given address.
    pub fn balance_of(&self, owner: Address) -> Balance {
        *self.balances.get(&owner).unwrap_or(&0)
	}

	/// Returns the amount of tokens that an owner allowed to a spender.
	pub fn allowance(&self, owner: Address, spender: Address) -> Balance {
		// let pair = AllowancePair{owner, spender};
		*self.allowances.get(&(owner, spender)).unwrap_or(&0)
	}

	/// Transfers token from the sender to the `to` address.
	pub fn transfer(&mut self, to: Address, value: Balance) -> bool {
		self.transfer_impl(ContractEnv::caller(), to, value);
		true
	}

	/// Approve the passed address to spend the specified amount of tokens
	/// on the behalf of the message's sender.
	///
	/// # Note
	///
	/// Beware that changing an allowance with this method afterwards brings
	/// the risk that someone may use both, the old and the new allowance,
	/// by unfortunate transaction ordering.
	/// One possible solution to mitigate this race condition is to first reduce
	/// the spender's allowance to 0 and set the desired value afterwards:
	/// https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
	pub fn approve(&mut self, spender: Address, value: Balance) -> bool {
		assert_ne!(spender, zero_address());
		let owner = ContractEnv::caller();
		// let pair = AllowancePair{owner, spender};
		self.allowances.insert((owner, spender), value);
		// emit event (not ready yet)
		true
	}

	/// Transfer tokens from one address to another.
	///
	/// Note that while this function emits an approval event,
	/// this is not required as per the specification,
	/// and other compliant implementations may not emit the event.
	pub fn transfer_from(&mut self, from: Address, to: Address, value: Balance) -> bool {
		// let pair = AllowancePair{owner: from, spender: to};
		// self.allowances.mutate_with(&pair, |allowed| *allowed -= value);
		self.allowances[&(from, to)] -= value;
		self.transfer_impl(from, to, value);
		// emit approval(from, to, value) (not yet ready)
		true
	}

	/// Transfers token from a specified address to another address.
	fn transfer_impl(&mut self, from: Address, to: Address, value: Balance) {
		assert_ne!(to, zero_address());

		self.balances[&from] -= value;
		self.balances[&to] += value;

		// emit transfer(from, to, value) (not ready yet)
	}
}

#[no_mangle]
pub extern "C" fn deploy() {}

#[no_mangle]
pub extern "C" fn call() {
	// use parity_codec::{Decode};
	// use pdsl_core::{
	// 	env::{Env, ContractEnv},
	// };

	// let input = ContractEnv::input();
	// let action = Action::decode(&mut &input[..]).unwrap();

	// let mut subpeep = crate::Subpeep::default();
	// match action {
	// 	Action::Register{username} => {
	// 		subpeep.register(&username);
	// 	}
	// 	Action::PeepMessage{username, message} => {
	// 		subpeep.peep_message(&username, &message)
	// 	}
	// 	Action::Follow{following, followed} => {
	// 		subpeep.follow(&following, &followed)
	// 	}
	// }
}
