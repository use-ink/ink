# Storage refactoring

In ink! v4 the way storage works was refactored.

## ink! v4 storage

First of all, new version of ink!'s storage substantially changes
the way you can interact with "spread structs" (structs that span multiple
storage cells, for which you had to use `SpreadLayout` in previous versions of ink!)
by allocating storage keys in compile-time.

For example, consider the previous struct with `SpreadLayout` derived:

```rust
#[derive(SpreadLayout)]
struct TestStruct {
    first: Mapping<u32, u32>,
    second: Mapping<u64, u64>
}
```

With new ink! version, it looks like this:

```rust
#[ink::storage_item]
struct TestStruct {
    first: Mapping<u32, u32>,
    second: Mapping<u64, u64>
}
```

The compiler will automatically allocate storage keys for your fields,
without relying on fields iteration like in the previous ink! version.

With these changes, `SpreadLayout` trait was removed, and methods like `pull_spread` and `push_spread` are now unavailable.

A new trait, `Storable`, was introduced instead. It represents types that can be read and written into the contract's storage. Any type that implements `scale::Encode` and `scale::Decode`
automatically implements `Storable`.

You can also use `#[ink::storage_item]` to automatically implement `Storable`
and make [your struct](https://use.ink/datastructures/custom-datastructure#using-custom-types-on-storage) fully compatible with contract's storage. This attribute
automatically implements all necessary traits and calculates storage keys for types.
You can also set `#[ink::storage_item(derive = false)]` to remove auto-derive
and derive everything manually later:

```rust
#[ink::storage_item]
struct MyNonPackedStruct {
    first_field: u32,
    second_field: Mapping<u32, u32>,
}

#[ink::storage_item(derive = false)]
#[derive(Storable, StorableHint, StorageKey)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
struct MyAnotherNonPackedStruct {
    first_field: Mapping<u128, Vec<u8>>,
    second_field: Mapping<u32, u32>,
}
```

For [precise storage key configuration](https://use.ink/datastructures/storage-layout#manual-vs-automatic-key-generation) several new types were introduced:

* `StorableHint` is a trait that describes the stored type, and its storage key.
* `ManualKey` is a type, that describes the storage key itself. You can, for example,
set it to a custom value - `ManualKey<123>`.
* `AutoKey` is a type, that gets automatically replaced with the `ManualKey` with
compiler-generated storage key.

For example, if you want to use the `Mapping`, and you want to set the storage key manually, you can take a look at the following example:

```rust
#[ink::storage_item]
struct MyStruct {
    first_field: u32,
    second_field: Mapping<u32, u32, ManualKey<123>>,
}
```

For [packed structs](https://use.ink/datastructures/storage-layout#packed-vs-non-packed-layout), a new trait was introduced - `Packed`. It represents structs,
all fields of which occupy a single storage cell. Any type that implements
`scale::Encode` and `scale::Decode` receives a `Packed` implementation:

Unlike non-packed types created with `#[ink::storage_item]`, packed types don't have
their own storage keys.

```rust
#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
struct MyPackedStruct {
    first_field: u32,
    second_field: Vec<u8>,
}
```

Example of nested storage types:

```rust
#[ink::storage_item]
struct NonPacked {
    s1: Mapping<u32, u128>,
    s2: Lazy<u128>,
}

#[derive(scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
struct Packed {
    s1: u128,
    s2: Vec<u128>,
}

#[ink::storage_item]
struct NonPackedComplex<KEY: StorageKey> {
    s1: (String, u128, Packed),
    s2: Mapping<u128, u128>,
    s3: Lazy<u128>,
    s4: Mapping<u128, Packed>,
    s5: Lazy<NonPacked>,
    s6: PackedGeneric<Packed>,
    s7: NonPackedGeneric<Packed>,
}
```

Every non-packed type also has `StorageKey` trait implemented for them. This trait is used for calculating storage key types.

There also exists a way to use `StorageKey` for types that are packed - you can just use `Lazy`, a wrapper around type
which allows to store it in [separate storage cell under it's own storage key](https://use.ink/datastructures/storage-layout#eager-loading-vs-lazy-loading). You can use it like this:

```rust
#[ink::storage_item]
struct MyStruct {
    first_field: Lazy<u32>,
    second_field: Mapping<u32, u32>,
}
```

In this case, `first_field` will be stored in it's own storage cell.

If you add generic that implements `StorageKey` to your type, it will be used as a storage key for this type, otherwise it will be 
set to `AutoKey`. For example this struct has its storage key automatically derived by the compiler:

```rust
#[ink::storage_item]
struct MyStruct {
    first_field: u32,
    second_field: Mapping<u32, u32>,
}
```

On the other hand, you can manually set storage key offset for your struct. This offset will apply to every non-packed field in a struct:

```rust
#[ink::storage_item]
struct MyStruct<KEY: StorageKey> {
    first_field: u32,
    second_field: Mapping<u32, u32, ManualKey<123>>,
}
```

When your struct has a `KEY` generic existing, the `#[ink::storage_item]` macro will automatically set
the `ParentKey` generic value to `KEY`, basically concatenating two values together.

The reason to do it in such way is that you can use the same type in different places and set different storage keys for them.

For example if you want to use it in contract, you can do it like this:

```rust
#[ink(storage)]
struct MyContract {
    my_struct: MyStruct<ManualKey<123>>,
}
```

or

```rust
#[ink(storage)]
struct MyContract {
    my_struct: MyStruct<AutoKey>,
}
```

After that, if you try to assign the new value to a field of this type, you will get an error, because after code generation,
it will be another type with generated storage key:

```rust
#[ink(constructor)]
pub fn new() -> Self {
    let mut instance = Self::default();

    instance.balances = Balances::<ManualKey<123>>::default();

    instance
}
```

You will get an error that look similar to this:

```shell
note: expected struct `Balances<ResolverKey<ManualKey<_, _>, ManualKey<4162912002>>>`
found struct `Balances<ManualKey<_, _>>`
```

That's so, because every type is unique and has it's own storage key after code generation.

So, the way to fix it is to use `Default::default()` so it will generate right type:

```rust
instance.balances = Default::default();
```

### Caveats

There is a known problem with generic fields that are non-packed in structs. Example:

```rust
#[ink::storage_item]
struct MyNonPackedStruct<D: MyTrait = OtherStruct> {
    first_field: u32,
    second_field: D,
}

struct OtherStruct {
    other_first_field: Mapping<u128, u128>,
    other_second_field: Mapping<u32, Vec<u8>>,
}

trait MyTrait {
    fn do_something(&self);
}

impl MyTrait for OtherStruct {
    fn do_something(&self) {
        // do something
    }
}
```

In this case contract cannot be built because it cannot calculate the storage key for the field `second_field` of type `MyTrait`.

You can use packed structs for it or, as a temporary solution, set `ManualKey` as another trait for field:

```rust
struct MyNonPackedStruct<D: MyTrait + ManualKey<123> = OtherStruct>
```

But instead of a `ManualKey<123>` you should use key that was generated during compilation. Packed generics work okay, so you can use it like this:

```rust
#[ink::storage_item]
struct MyNonPackedStruct<D: Packed> {
    first_field: u32,
    second_field: D,
}
```

You should also check the [ink! storage layout documentation](https://use.ink/datastructures/storage-layout#considerations) for more
details on known caveats and considerations.
