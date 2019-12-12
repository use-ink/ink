// Copyright 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A BTreeMap collection.
//!
//! This implementation follows the algorithm used by the Rust BTreeMap stdlib
//! implementation. The Rust implementation was in general the blueprint for
//! this implementation. The major difference is that the Rust implementation
//! is in-memory, whereas this implementation uses the ink! primitives for storage.
//!
//! The idea of a BTreeMap is to store many elements (i.e. key/value pairs)
//! in one tree node. Each of these elements can have a left and right child.
//! A simple node with three elements thus can look like this:
//!
//! ```no_compile
//! keys  = [    a,    b,    c    ];
//! vals  = [    a,    b,    c    ];
//! edges = [ 1,    2,    3,    4 ];
//! ```
//!
//! Here the left child of element `a` would be the node with the index `1`, its
//! right child the node with index `2`.
//!
//! This concept of multiple elements stored in one node suits our needs well,
//! since expensive storage fetches are reduced.
//!
//! For a description of the tree algorithm itself it's best to see the merge/split
//! method comments. A notable thing is that the algorithm will merge nodes if it's
//! possible to reduce storage space (see `handle_underfull_nodes()` for more info).

#[cfg(all(test, feature = "test-env"))]
mod tests;

mod impls;
mod search;

pub use self::impls::{
    BTreeMap,
    Entry,
};
