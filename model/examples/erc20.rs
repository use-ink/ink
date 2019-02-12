use crate::{
	contract::ContractDecl,
	state::ContractState,
	msg::Message,
};
use pdsl_core::{
	env::{
		Env,
		ContractEnv,
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
		total: storage::Value<Balance>,
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

fn instantiate() -> impl ContractInstance {
	Contract::new::<Erc20Token>()
		.on_deploy(|env, init_supply| {
			env.state.balances[ContractEnv::caller()] = init_supply;
			env.state.total.set(init_supply);
		})
		.on_msg::<TotalSupply>(|env, _| {
			*env.state.total.get()
		})
		.on_msg::<BalanceOf>(|env, owner| {
			env.state.balances[owner]
		})
		.on_msg_mut::<Transfer>(|env, (to, amount)| {
			if amount == Address::from(0x0) {
				return false;
			}
			let from = env.caller();
			let balance_from = env.state.balances[from];
			let balance_to = env.state.balances[to];

			if balance_from >= amount {
				env.state.balances[from] = existing_from - amount;
				env.state.balances[to] = existing_to + amount;
				return true
			}

			false
		});
}

#[no_mangle]
fn deploy() {
	instantiate().deploy()
}

#[no_mangle]
fn call() {
	instantiate().run()
}
