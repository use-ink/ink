# Storage refactoring

In ink! v4.0.0-beta the way storage works was refactored.

Previously each field was stored in its storage cell under its storage key.
The storage key was calculated in runtime over fields iteration. In a new version, all packed fields are stored in one
storage cell under one storage key. All non-packed fields know their storage key during compilation time.
During compilation time for types are generated their own storage keys, so, for instance, if you have `Mapping<u128, 128>`
type for your storage field, after code generation it will look like `Mapping<u128, u128, ManualKey<123>>` which is not the same
type as just `Mapping<u128, u128>`.

### Traits

First of all, removed traits `SpreadLayout` and `PackedLayout`. Now, instead `pull_spread`, `push_spread` and other methods
that were used in these traits you should use `ink::env::set_contract_storage` and `ink::env::get_contract_storage`.
Instead of this exist traits `Storable` and `Packed`. Also, you can use macro attribute `#[ink::storage_item]` to implement this traits

 - `Storable` is a trait that is created for representing types that can be read and written into the contract's storage.
Types that implement `scale::Decode` and `scale::Encode` are storable by default.

You can derive `Storable` trait for your type as in the following example:
```rust
use ink::storage::traits::Storable;

#[derive(Storable)]
struct MyStruct {
    first_field: u32,
    second_field: Vec<u8>,
}
```

 - `Packed` is a trait that is created for representing types that can be read and written into the contract's storage, 
and all of it's fields occupy a single storage cell.

You can derive `Packed` trait for your type as in the following example:

```rust
#[derive(scale::Encode, scale::Decode)]
struct MyPackedStruct {
    first_field: u32,
    second_field: Vec<u8>,
}
```

A type will be considered non-packed if any of it's fields occupies it's single storage cell. Example of non-packed type:

```rust
#[ink::storage_item]
struct MyNonPackedStruct {
    first_field: u32,
    second_field: Mapping<u32, u32>,
}
```
The type is considered non-packed as `Mapping` always occupies it's own storage cell.

Packed types don't have their storage keys, but non-packed types require calculating the storage key during compilation.

 - `#[ink::storage_item]` macro attribute prepares your type to be fully compatible and usable with a storage.
It implements all necessary traits and calculates the storage key for types. You can set `#[ink::storage_item(derive = false)]` that will indicate that will disable auto-deriving of all required traits.
Also, it will be implemented via blanket implementation for every type that implements `scale::Encode` and `scale::Decode`. Following examples show how to create
type using `ink::storage_item`, `scale::Encode` and `scale::Decode`.
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

- `StorableHint` is a trait that describes the type that should be used for storing the value and preferred storage key. 
It is implemented automatically for all types that implement `Packed` types.
- `ManualKey` is a generic struct that is used if you want to set the storage key manually, so you can use it like `ManualKey<123>`.
- `AutoKey` is a struct that is used if you want to set the storage key automatically, so you can use it like `AutoKey`.
In this case, the storage key will be calculated while compilation and set as `ManualKey<>` with calculated storage key.

For example, if you want to use the mapping, and you want to set the storage key manually, you can take a look at the following example:
```rust
#[ink::storage_item]
struct MyStruct {
    first_field: u32,
    second_field: Mapping<u32, u32, ManualKey<123>>,
}
```

- `StorageKey` is a trait that is used for calculating the storage key for types. It is implemented automatically for all types that are non-packed.

There also exists way to use `StorageKey` for types that are packed - you can just use `Lazy`, that is a wrapper around type,
which allows to store it in separate storage cell under it's own storage key. You can use it like this:
```rust

#[ink::storage_item]
struct MyStruct {
    first_field: Lazy<u32>,
    second_field: Mapping<u32, u32>,
}
```

In this case, `first_field` will be stored in it's own storage cell.

If you add generic that implements `StorageKey` to your type, it will be used as a storage key for this type, otherwise it will be 
set to `AutoKey`. For example:
```rust
#[ink::storage_item]
struct MyStruct {
    first_field: u32,
    second_field: Mapping<u32, u32>,
}
```
In this case storage key will be calculated automatically. But if you want to offset the key, you can do it like this:
```rust
#[ink::storage_item]
struct MyStruct<KEY: StorageKey> {
    first_field: u32,
    second_field: Mapping<u32, u32, ManualKey<123>>,
}
```

In this case, you can set storage key that you prefer for instances of this type.

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

### Problems
There is a problem with generic fields that are non-packed in structs. Example:
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

But instead of `ManualKey<123>` you should use key that was generated during compilation. Packed generics work okay, so you can use it like this:
```rust
#[ink::storage_item]
struct MyNonPackedStruct<D: Packed> {
    first_field: u32,
    second_field: D,
}
```
