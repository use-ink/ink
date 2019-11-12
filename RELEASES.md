# Version 2.0 Syntax (2019-11-11)

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

## Declaring a Contract

The fundamental change with the new ink! syntax is how we declare a new contract.

We used to wrap the whole ink! contract into a `contract!` macro. At that point, all syntax within
the macro could be custom, and mistakenly we used that to define custom syntax that made our code
not really Rust anymore.

Now we wrap the whole contract in a standard Rust module, and include an attribute tag to identify
this object as part of the ink! language. This means that all of our code from this point forward
will be valid Rust!

### Before

```rust
contract! {
    ...
}
```

### After

```rust
#[ink::contract(version = "0.1.0")]
mod erc20 {
    ...
}
```

> Note, we now require a mandatory ink! version in the header. You're welcome.

## ink! Contract Tag

The ink! contract tag can be extended to provide other configuration information about your
contract.

### Defining Custom Types

We used to define types using a special `#![env = DefaultSrmlTypes]` tag.

Now we simply include the type definition in the `#[ink::contract(...)]` tag:

```rust
#[ink::contract(version = "0.1.0", types = MyCustomTypes)]
```

By default, we use `DefaultSrmlTypes`, so you don't need to define anything unless you plan to use
custom types.

### Dynamic Allocation

It is possible to enable the dynamic environment that allows for dynamic allocations by specifying
`dynamic_allocations = true` in the parameters of the ink! header. This is disabled by default.

```rust
#[ink::contract(version = "0.1.0", dynamic_allocations = true)]
```

## Declaring Storage

We define storage items just the same as before, but now we need to add the `#[ink(storage)]`
attribute tag.

### Before

```rust
struct Erc20 {
    total_supply: storage::Value<Balance>,
    balances: storage::HashMap<AccountId, Balance>,
    allowances: storage::HashMap<(AccountId, AccountId), Balance>,
}
```

### After

```rust
#[ink(storage)]
struct Erc20 {
    total_supply: storage::Value<Balance>,
    balances: storage::HashMap<AccountId, Balance>,
    allowances: storage::HashMap<(AccountId, AccountId), Balance>,
}
```

## Declaring Events

To update your events, you need to:

1. Change the old `event` keyword to a standard Rust `struct`.
2. Add the `#[ink(event)]` attribute tag to your `struct`.
3. Add the `#[ink(topic)]` attribute tag to each item in your event.

### Before

```rust
event Transfer {
    from: Option<AccountId>,
    to: Option<AccountId>,
    value: Balance,
}
```

### After

```rust
#[ink(event)]
struct Transfer {
    #[ink(topic)]
    from: Option<AccountId>,
    #[ink(topic)]
    to: Option<AccountId>,
    #[ink(topic)]
    value: Balance,
}
```

## Environment Handler

`EnvHandler` is no longer exposed to the user and instead the environment is now always accessed via
`self.env()`.

### Before

**Getting the caller:**

```rust
let caller = env.caller();
```

**Emitting an event:**

```rust
env.emit(...)
```

### After

**Getting the caller:**

```rust
let caller = self.env().caller();
```

**Emitting an event:**

```rust
self.env().emit_event(...)
```

> Note: The name of the function used to emit an event was updated to `emit_event`.

## Message Functions

We used to use `pub(external)` to tag functions that could be called by the outside world.

We now simply add the attribute `#[ink(message)]`.

### Before

```rust
pub(external) fn total_supply(&self) -> Balance {
    *self.total_supply
}
```

### After

```rust
#[ink(message)]
fn total_supply(&self) -> Balance {
    *self.total_supply
}
```

## Defining a Constructor

We used to define our constructor by implementing the `Deploy` trait and defining the `deploy`
function.

But now our constructor function is in the same place as the rest of our contract functions, within
the general implementation of the storage struct.

We tag these functions with the `#[ink(constructor)]` attribute. We can create multiple different
constructors by simply creating more functions with the same tag. You can name a constructor
function whatever you want (except starting with `__ink` which is reserved for all functions).

<table>
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

## Cross Contract Calls

It is now possible to call ink! messages and ink! constructors. So ink! constructors allow
delegation and ink! messages can easily call other ink! messages.

Given another ink! contract like `mod Accumulator { ... }`, we can call any of its functions just
like we would expect:

```rust
use accumulator::Accumulator;
let result = Accumulator::my_function()
```

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

### Before

```rust
let accumulator = Accumulator::new(accumulator_code_hash, init_value)
    .value(total_balance / 4)
    .create()
    .expect("failed at instantiating the accumulator contract");
```

### After

```rust
let accumulator = Accumulator::new(init_value)
    .value(total_balance / 4)
    .using_code(accumulator_code_hash)
    .create_using(self.env())
    .expect("failed at instantiating the `Accumulator` contract");
```

## Contract Tests

Testing contracts off-chain is done by `cargo test` and users can simply use the standard routines
of creating unit test modules within the ink! module itself:

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

Messages can simply be called on the returned instance as if MyContract::my_constructor returns a
Self instance.

**The off-chain test environment has lost a bit of power compared to the old ink! language.**

It is not currently possible to query and set special test data about the environment (such as the
caller of a function or amount of value sent), but these will be added back in the near future.

## ink!-less Implementations

It is also possible to annotate an entire impl blocks with:

```rust
#[ink(impl)]
impl Something { ... }.
```

This is useful if the `impl` block itself doesn't contain any ink! constructors or messages. This
makes the ink! compiler aware that this is still an ink! definition.

It should only be used in this situation.