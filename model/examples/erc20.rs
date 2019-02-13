#![feature(const_str_as_bytes)]

use pdsl_model::{
	ContractDecl,
	Contract,
	state,
	messages,
};
use pdsl_core::{
	env::{
		srml::Address,
		srml::Balance,
	},
	storage,
};

state! {
	/// A simple implementation of a rudimentary Erc20 token contract.
	struct Erc20Token {
		/// The balance for an address.
		balances: storage::HashMap<Address, Balance>,
		/// The total supply.
		total: storage::Value<Balance>
	}
}

messages! {
	/// Returns the total supply.
	TotalSupply() -> Balance;
	/// Returns the balance of the given address.
	BalanceOf(owner: Address) -> Balance;
	/// Transfers balance from the caller to the given address.
	///
	/// Returns `true` if the transfer was successful.
	Transfer(to: Address, amount: Balance) -> bool;
}

fn instantiate() -> impl Contract {
	ContractDecl::new::<Erc20Token>()
		.on_deploy(|env, init_supply| {
			let caller = env.caller();
			env.state.balances[&caller] = init_supply;
			env.state.total.set(init_supply);
		})
		.on_msg::<TotalSupply>(|env, _| {
			*env.state.total.get()
		})
		.on_msg::<BalanceOf>(|env, owner| {
			env.state.balances[&owner]
		})
		.on_msg_mut::<Transfer>(|env, (to, amount)| {
			// if amount == Address::from(0x0) { // In Substrate we do not have the zero address!
			//	 return false;
			// }
			let from = env.caller();
			let balance_from = env.state.balances[&from];
			let balance_to = env.state.balances[&to];

			if balance_from >= amount {
				env.state.balances[&from] = balance_from - amount;
				env.state.balances[&to] = balance_to + amount;
				return true
			}

			false
		})
		.instantiate()
}

#[no_mangle]
fn deploy() {
	instantiate().deploy()
}

#[no_mangle]
fn call() {
	instantiate().dispatch()
}
