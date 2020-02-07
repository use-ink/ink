# Version 2.0 Syntax (2019-12-03)

The ink! version 2.0 syntax has one major philosophy:

> Just. Be. Rust.

To accomplish this, we take advantage of all the standard Rust types and structures and use
[attribute macros](https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros) to
tag these standard structures to be different parts of the ink! language.

Anything that is not tagged with an `#[ink(...)]` attribute tag is just standard Rust, and can be
used in and out of your contract just like standard Rust could be used!

**Every valid ink! contract is required to have at least one `#[ink(constructor)]`, at least one
`#[ink(message)]` and exactly one `#[ink(storage)]` attribute.**

Follow the instructions below to understand how to migrate your ink! 1.0 contracts to this new ink!
2.0 syntax.

## Update the ink! CLI

Install the latest ink! CLI using the following command:

```bash
cargo install --git https://github.com/paritytech/cargo-contract cargo-contract --force
```

There is a new contract metadata format you need to use. You can generate the metadata using:

```bash
cargo contract generate-metadata
```

This will generate a file `metadata.json` you should upload when deploying or interacting with a
contract.

## Declaring a Contract

The fundamental change with the new ink! syntax is how we declare a new contract.

We used to wrap the whole ink! contract into a `contract!` macro. At that point, all syntax within
the macro could be custom, and in our first iteration of the language, we used that in ways that
made our code not really Rust anymore.

Now we wrap the whole contract in a standard Rust module, and include an attribute tag to identify
this object as part of the ink! language. This means that all of our code from this point forward
will be valid Rust!

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
contract! {
    ...
}
```

</td>
<td>

```rust
use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod erc20 {
    ...
}
```

</td>
</tr>
</table>

> Note: we now require a mandatory ink! version in the header. You're welcome.

See the [ERC20 example](https://github.com/paritytech/ink/blob/master/examples/erc20/src/lib.rs).

## ink! Contract Tag

The ink! contract tag can be extended to provide other configuration information about your
contract.

### Defining Custom Types

We used to define types using a special `#![env = DefaultSrmlTypes]` tag.

Now we simply include the type definition in the `#[ink::contract(...)]` tag:

```rust
#[ink::contract(version = "0.1.0", env = MyCustomTypes)]
```

By default, we use `DefaultSrmlTypes`, so you don't need to define anything unless you plan to use
custom types.

### Dynamic Allocation

It is possible to enable the dynamic environment that allows for dynamic allocations by specifying
`dynamic_allocations = true` in the parameters of the ink! header. This is disabled by default.

```rust
#[ink::contract(version = "0.1.0", dynamic_allocations = true)]
```

> Note: The dynamic environment is still under research and not yet stable.

## Declaring Storage

We define storage items just the same as before, but now we need to add the `#[ink(storage)]`
attribute tag.

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
struct Erc20 {
    total_supply: storage::Value<Balance>,
    balances: storage::HashMap<AccountId, Balance>,
    allowances: storage::HashMap<(AccountId, AccountId), Balance>,
}
```

</td>
<td>

```rust
#[ink(storage)]
struct Erc20 {
    total_supply: storage::Value<Balance>,
    balances: storage::HashMap<AccountId, Balance>,
    allowances: storage::HashMap<(AccountId, AccountId), Balance>,
}
```

</td>
</tr>
</table>

See the [ERC20 example](https://github.com/paritytech/ink/blob/master/examples/erc20/src/lib.rs).

## Declaring Events

To update your events, you need to:

1. Change the old `event` keyword to a standard Rust `struct`.
2. Add the `#[ink(event)]` attribute tag to your `struct`.

If you were previously indexing the items in your event with `#[indexed]`:

3. Add the `#[ink(topic)]` attribute tag to each item in your event.

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
event Transfer {
    from: Option<AccountId>,
    to: Option<AccountId>,
    #[indexed]
    value: Balance,
}
```

</td>
<td>

```rust
#[ink(event)]
struct Transfer {
    from: Option<AccountId>,
    to: Option<AccountId>,
    #[ink(topic)]
    value: Balance,
}
```

</td>
</tr>
</table>

See the [ERC20 example](https://github.com/paritytech/ink/blob/master/examples/erc20/src/lib.rs).

## Environment Handler

`EnvHandler` is no longer exposed to the user and instead the environment is now always accessed via
`self.env()`.

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

**Getting the caller:**

```rust
let caller = env.caller();
```

**Emitting an event:**

```rust
env.emit(...)
```

</td>
<td>

**Getting the caller:**

```rust
let caller = self.env().caller();
```

**Emitting an event:**

```rust
self.env().emit_event(...)
```

</td>
</tr>
</table>

> Note: The name of the function used to emit an event was updated to `emit_event`.

## Message Functions

We used to use `pub(external)` to tag functions that could be called by the outside world.

We now simply add the attribute `#[ink(message)]`.

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
pub(external) fn total_supply(&self) -> Balance {
    *self.total_supply
}
```

</td>
<td>

```rust
#[ink(message)]
fn total_supply(&self) -> Balance {
    *self.total_supply
}
```

</td>
</tr>
</table>

See the [ERC20 example](https://github.com/paritytech/ink/blob/master/examples/erc20/src/lib.rs).

## Defining a Constructor

We used to define our constructor by implementing the `Deploy` trait and defining the `deploy`
function.

But now our constructor function is in the same place as the rest of our contract functions, within
the general implementation of the storage struct.

We tag these functions with the `#[ink(constructor)]` attribute. We can create multiple different
constructors by simply creating more functions with the same tag. You can name a constructor
function whatever you want (except starting with `__ink` which is reserved for all functions).

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

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
}
```

</td>
</tr>
</table>

See the [ERC20 example](https://github.com/paritytech/ink/blob/master/examples/erc20/src/lib.rs).

## Cross Contract Calls

It is now possible to call ink! messages and ink! constructors. So ink! constructors allow
delegation and ink! messages can easily call other ink! messages.

Given another ink! contract like `mod Adder { ... }`, we can call any of its functions:

```rust
use adder::Adder;
//--snip--
#[ink(storage)]
struct Delegator {
    adder: storage::Value<Adder>,
}
//--snip--
let result = self.adder.inc(by);
```

See the [delegator example](https://github.com/paritytech/ink/blob/master/examples/delegator/lib.rs).

## Factory Contracts

Creation of other contracts from a factory contract works pretty much the same way it did in the old
ink! language.

However, users are now required to specify the `code_hash` separately rather than in the
constructor:

```rust
.using_code(code_hash)
```

Also, they need to specify the used ink! environment (most likely `self.env()`):

```rust
create_using(self.env())
```

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
let accumulator = Accumulator::new(accumulator_code_hash, init_value)
    .value(total_balance / 4)
    .create()
    .expect("failed at instantiating the accumulator contract");
```

</td>
<td>

```rust
let accumulator = Accumulator::new(init_value)
    .value(total_balance / 4)
    .gas_limit(12345)
    .using_code(accumulator_code_hash)
    .create_using(self.env())
    .expect("failed at instantiating the `Accumulator` contract");
```

</td>
</tr>
</table>

See the [delegator example](https://github.com/paritytech/ink/blob/master/examples/delegator/lib.rs).

## Contract Tests

Testing contracts off-chain is done by `cargo test` and users can simply use the standard routines
of creating unit test modules within the ink! project:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_test() { ... }
}
```

Test instances of contracts can be created with something like:

```rust
let contract = MyContract::my_constructor(a, b);
```

Messages can simply be called on the returned instance as if `MyContract::my_constructor` returns a
`Self` instance.

See the [flipper example](https://github.com/paritytech/ink/blob/master/examples/flipper/src/lib.rs).

**The off-chain test environment has lost a bit of power compared to the old ink! language.**

It is not currently possible to query and set special test data about the environment (such as the
caller of a function or amount of value sent), but these will be added back in the near future.

## ink!-less Implementations

It is also possible to annotate an entire impl blocks with:

```rust
#[ink(impl)]
impl Contract {
    fn internal_function(&self) {
        self.env().emit_event(EventName);
    }
}.
```

This is useful if the `impl` block itself doesn't contain any ink! constructors or messages, but you
still need to access some of the "magic" provided by ink!. In the example above, you would not have
access to `emit_event` without `#[ink(impl)]`.
