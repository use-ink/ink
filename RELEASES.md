# Version 2.0 Syntax (2019-11-11)


The ink! contract module allows Rust item definitions as well as ink! definitions annotated by #[ink(..)] markers


Users can now specify multiple constructors that are simple &mut self methods annotated with #[ink(constructor)]

Defining a message is now done by annotating it with #[ink(message)]

Defining an event is now done by annotating a Rust struct with #[ink(event)]

Event topics are declared by annotating the event parameter in question with #[ink(topic)] (currently ignored though)

It is also possible to annotate whole impl blocks with #[ink(impl)]. This is useful if the impl block itself doesn't contain any ink! constructors or messages so ink! compiler is aware that this is still an ink! definition. Has to be used only in this situation. We should actually add warnings if used while having constructors or messages as well.
Cross-calling of contracts works pretty much the same way it did in the old ink! language, however users are now required to specify the code_hash (.using_code(my_code_hash)) separately and also need to specify the used ink! environment (create_using(self.env()) normally).
It is now possible to call ink! messages and ink! constructors. So ink! constructors allow delegation and ink! messages can easily call other ink! messages.
EnvHandler is no longer exposed to the user and instead the environment is now always accessed via self.env() is all ink! constructors, messages and private methods.
It is possible to enable the dynamic environment that allows for dynamic allocations by specifying dynamic_allocations = true in the parameters of the ink! header. This is disabled by default.
Testing contracts off-chain is done by cargo test and users can simply use the standard routines of creating unit test modules within the ink! module itself (#[cfg(test)] mod tests { use super::*; #[test] fn my_test() { ... } }). Test instances of contracts can be created with MyContract::my_constructor(a, b) if the contract has an #[ink(storage)] struct MyContract { ... } and an #[ink(constructor)] fn my_constructor(a: A, b: B) { ... }. Messages can simply be called on the returned instance as if MyContract::my_constructor returns a Self instance.
The off-chain test environment has lost a bit of power compared to the old ink! lang. It currently is no longer possible to query and set special test data about the environment but these restirctions are going to be lifted in the future.

<table>
<tr>
<th>Feature</th>
<th>v1.0 Syntax</th>
<th>v2.0 Syntax</th>
</tr>
<tr>
<td>
Declaring a Contract
</td>
<td>

We used to use a single `contract!` macro:

```rust
contract! {
	...
}
```
</td>
<td>

We now use `#[ink::contract(..)]` attribute macros on Rust modules:

```rust
#[ink::contract(version = "0.1.0")]
mod erc20 {
	...
}
```

> Note, we now require a mandatory ink! version in the header.
</td>
</tr>
<tr>
<td>
Defining Types
</td>
<td>
We used to define types using a special tag like this:

```rust
#![env = DefaultSrmlTypes]
```
</td>
<td>

This is now automatically defined in your `ink::contract` attribute, but can also be custom defined like:

```rust
#[ink::contract(version = "0.1.0", )]
```

TODO

</td>
</tr>
<tr>
<td>
Declaring Storage
</td>
<td>

We defined storage items in a single `struct`:

```rust
struct Erc20 {
	total_supply: storage::Value<Balance>,
	balances: storage::HashMap<AccountId, Balance>,
	allowances: storage::HashMap<(AccountId, AccountId), Balance>,
}
```

</td>
<td>

We just need to add the attribute `ink(storage)`:

```rust
#[ink(storage)]  // <-- This
struct Erc20 {
	total_supply: storage::Value<Balance>,
	balances: storage::HashMap<AccountId, Balance>,
	allowances: storage::HashMap<(AccountId, AccountId), Balance>,
}
```
</td>
</tr>

<tr>
<td>
Defining a Constructor
</td>
<td>

We used to define our constructor by implementing `Deploy` and the `deploy` function:

```rust
impl Deploy for Erc20 {
	fn deploy(&mut self, init_supply: Balance) {
		let caller = env.caller();
		self.total_supply.set(init_value);
		self.balances.insert(caller, init_supply);
		env.emit(Transfer {
			from: None,
			to: Some(env.caller()),
			value: init_value
		});
	}
}
```
</td>
<td>
We can now create multiple different constructors by simply creating functions with the correct tag:

```rust
impl Erc20 {
	#[ink(constructor)]
	fn new(&mut self, initial_supply: Balance) {
		let caller = self.env().caller();
		self.total_supply.set(initial_supply);
		self.balances.insert(caller, initial_supply);
		self.env().emit_event(Transferred {
			from: None,
			to: Some(caller),
			amount: initial_supply,
		});
	}
//-snip-
```

You can name them whatever you want.

> Note: We define the constructor function in the same place we define other contract functions.
</td>
</tr>
<tr>
<td>
Message Functions
</td>
<td>

We used to use `pub(external)` to tag functions that could be called by the outside world.

```rust
pub(external) fn total_supply(&self) -> Balance {
	*self.total_supply
}
```
</td>
<td>

We now simply add the attribute `ink(message)`:

```rust
#[ink(message)]
fn total_supply(&self) -> Balance {
	*self.total_supply
}
```
</td>
</tr>
<tr>
<td>
</td>
<td>
</td>
<td>
</td>
</tr>
<tr>
<td>
</td>
<td>
</td>
<td>
</td>
</tr>
<tr>
<td>
</td>
<td>
</td>
<td>
</td>
</tr>
<tr>
<td>
</td>
<td>
</td>
<td>
</td>
</tr>
</table>

