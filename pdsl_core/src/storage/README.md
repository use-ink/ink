# Storage

This module provides some low-level primitives to operate on contract storage.

Some primitives provide more or less guarantees to their users.
The following listing lists all the kinds of guarantees and what they actually provide for their users.

- `Typed`: If the entity is automatically encoded and decoded appropriately.
- `Owned`: If there can be only one owner at the same time for the respective entity.
- `Read Optimized`: Reads from contract storage are avoided if possible.
- `Safe Load`: Loading from contract storage will always yield an entity.

## Current System

The following table lists all provided types and the guarantees they support.

| Primitive | Typed | Owned | Read Optimized | Safe Load |
|:---------:|:-----:|:-----:|:--------------:|:---------:|
| `Key`     | No    | No    | No             | No        |
| `Stored`  | Yes   | No    | No             | No        |
| `Synced`  | Yes   | Yes   | No             | No        |
| `Cached`  | Yes   | Yes   | Yes            | No        |

## Future System

First, we might consider some renamings.

A `Key` is similar to a pointer in memory, so renaming it to something
more similar, like `Cell` could lead to less confusion.

For regularity all `*Cell` kinds may fail upon loading their elements
since they represent a contract storage cell that might be empty.

Also `Synced` and `Cached` got merged since it is possible and advantageous
for both kinds to avoid useless reads to the contract storage.

| Primitive    | Owned | Typed | Read Optimized | Safe Load |
|:------------:|:-----:|:-----:|:--------------:|:---------:|
| `Key`        | No    | No    | No             | No        |
| `Cell`       | Yes   | No    | No             | No        |
| `TypedCell`  | Yes   | Yes   | No             | No        |
| `CachedCell` | Yes   | Yes   | Yes            | No        |
| `Entity`     | Yes   | Yes   | Yes            | Yes       |

These different kinds can build on top of each other.
A `Cell` could be defined using `Key`, a `TypedCell` could be implemented using a `Cell` etc.
