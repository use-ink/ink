# Version 2.0 Syntax (2019-11-11)

The ink! version 2.0 syntax has one major philosophy:

* Just. Be. Rust.



The ink! contract module allows Rust item definitions as well as ink! definitions annotated by
#[ink(..)] markers





It is also possible to annotate whole impl blocks with #[ink(impl)]. This is useful if the impl
block itself doesn't contain any ink! constructors or messages so ink! compiler is aware that this
is still an ink! definition. Has to be used only in this situation. We should actually add warnings
if used while having constructors or messages as well.


Cross-calling of contracts works pretty much the same way it did in the old ink! language, however
users are now required to specify the code_hash (.using_code(my_code_hash)) separately and also need
to specify the used ink! environment (create_using(self.env()) normally).

It is now possible to call ink! messages and ink! constructors. So ink! constructors allow
delegation and ink! messages can easily call other ink! messages.

EnvHandler is no longer exposed to the user and instead the environment is now always accessed via
self.env() is all ink! constructors, messages and private methods.

It is possible to enable the dynamic environment that allows for dynamic allocations by specifying
dynamic_allocations = true in the parameters of the ink! header. This is disabled by default.

Testing contracts off-chain is done by cargo test and users can simply use the standard routines of
creating unit test modules within the ink! module itself (#[cfg(test)] mod tests { use super::*;
#[test] fn my_test() { ... } }). Test instances of contracts can be created with
MyContract::my_constructor(a, b) if the contract has an #[ink(storage)] struct MyContract { ... }
and an #[ink(constructor)] fn my_constructor(a: A, b: B) { ... }. Messages can simply be called on
the returned instance as if MyContract::my_constructor returns a Self instance.

The off-chain test environment has lost a bit of power compared to the old ink! lang. It currently
is no longer possible to query and set special test data about the environment but these
restirctions are going to be lifted in the future.


## Declaring a Contract

The biggest change with the new ink! syntax is that we used to wrap the whole ink! contract into a
`contract!` macro. At that point, all syntax within the macro could be custom, and we took advantage
of that custom syntax, which ultimately made our code not really Rust anymore.

Now we wrap the whole contract in a standard Rust `mod`, and include an attribute tag to identify
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

> Note, we now require a mandatory ink! version in the header.


## Defining Types

We used to define types using a special tag at the beginning of the `contract!` macro, but now we
simply include the type definition in the `ink::contract` tag. By default, we use
`DefaultSrmlTypes`, so you don't need to define anything unless you plan to use custom types.

You still have access to all the custom types throughout your contract.

### Before

```rust
contract! {
    #![env = DefaultSrmlTypes]
    //--snip--
}
```

### After

```rust
#[ink::contract(version = "0.1.0", types = DefaultSrmlTypes)]
```

> Note: `DefaultSrmlTypes` is included by default, but this may be useful if you are adding custom
> types.

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

## Defining a Constructor

We used to define our constructor by implementing the `Deploy` trait and defining the `deploy` function.

But now our constructor function is in the same place as the rest of our contract functions, within the general implementation of the storage struct.

We tag these functions with the `#[ink(constructor)]` attribute. We can create multiple different constructors by simply creating more functions with the same tag.

### Before

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

### After

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

## Declaring Events

To update your events, you need to:

2. Change the old `event` keyword to a standard Rust `struct`.
1. Add the `#[ink(event)]` attribute tag to your `struct`.
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

## Cross Contract Calls

Cross-calling of contracts works pretty much the same way it did in the old ink! language.

However, users are now required to specify the `code_hash` separately rather than in `new`:

```rust
.using_code(code_hash)
```

Also, they need to specify the used ink! environment (most likely `self.env()`):

```rust
create_using(self.env())
```

### Before

```rust
let accumulator = accumulator::Accumulator::new(accumulator_code_hash, init_value)
    .value(total_balance / 4)
    .create()
    .expect("failed at instantiating the accumulator contract");
```

### After

```rust
let accumulator = accumulator::Accumulator::new(init_value)
    .value(total_balance / 4)
    .using_code(accumulator_code_hash)
    .create_using(self.env())
    .expect("failed at instantiating the `Accumulator` contract");
```
