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

mod utils;

use utils::*;

use ink_primitives::Key;
use ink_core_derive::AllocateUsing;

#[derive(Default)]
struct DummyAlloc {
    allocated_cells: usize,
    allocated_chunks: usize,
}

impl ink_core::storage::alloc::Allocate for DummyAlloc {
    fn alloc(&mut self, size: u64) -> Key {
        if size == 1 {
            self.allocated_cells += 1;
        } else {
            self.allocated_chunks += 1;
        }
        Key([0x0; 32])
    }
}

#[derive(AllocateUsing, Debug, PartialEq, Eq)]
struct EmptyStruct;

#[derive(AllocateUsing, Debug, PartialEq, Eq)]
struct NewtypeStruct(Cell);

#[derive(AllocateUsing, Debug, PartialEq, Eq)]
struct NamedStruct {
    a: Cell,
    b: Chunk,
}

#[derive(AllocateUsing, Debug, PartialEq, Eq)]
struct ComplexNamedStruct {
    a: Chunk,
    b: Value<Cell>,
    c: Value<Chunk>,
}

#[derive(AllocateUsing, Debug, PartialEq, Eq)]
struct GenericStruct<T> {
    a: Cell,
    b: Chunk,
    c: Value<T>,
    d: Value<Value<T>>,
}

fn test_for<A>(expected_cells_alloc: usize, expected_chunks_alloc: usize)
where
    A: ink_core::storage::alloc::AllocateUsing,
{
    use ink_core::storage::alloc::AllocateUsing;
    let mut alloc = DummyAlloc::default();
    unsafe { <A as AllocateUsing>::allocate_using(&mut alloc) };
    assert_eq!(
        alloc.allocated_cells, expected_cells_alloc,
        "number of allocated cells doesn't match expected"
    );
    assert_eq!(
        alloc.allocated_chunks, expected_chunks_alloc,
        "number of allocated chunks doesn't match expected"
    );
}

fn main() {
    test_for::<EmptyStruct>(0, 0);
    test_for::<NewtypeStruct>(1, 0);
    test_for::<NamedStruct>(1, 1);
    test_for::<ComplexNamedStruct>(1, 2);
    test_for::<GenericStruct<Cell>>(3, 1);
    test_for::<GenericStruct<Chunk>>(1, 3);
}
