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

use ink_core::storage::Key;
use ink_core_derive::AllocateUsing;

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

struct Cell {}

impl ink_core::storage::alloc::AllocateUsing for Cell {
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: ink_core::storage::alloc::Allocate,
    {
        alloc.alloc(1);
        Self {}
    }
}

struct Chunk {}

impl ink_core::storage::alloc::AllocateUsing for Chunk {
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: ink_core::storage::alloc::Allocate,
    {
        alloc.alloc(100);
        Self {}
    }
}

struct Value<T> {
    value: T,
}

impl<T> ink_core::storage::alloc::AllocateUsing for Value<T>
where
    T: ink_core::storage::alloc::AllocateUsing,
{
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: ink_core::storage::alloc::Allocate,
    {
        Self {
            value: <T as ink_core::storage::alloc::AllocateUsing>::allocate_using(alloc),
        }
    }
}

#[derive(AllocateUsing)]
struct Single(Cell);

#[derive(AllocateUsing)]
struct B {
    a: Cell,
    b: Chunk,
}

#[derive(AllocateUsing)]
struct C {
    a: Chunk,
    b: Value<Cell>,
    c: Value<Chunk>,
}

#[derive(AllocateUsing)]
struct D<T> {
    a: Option<T>,
    b: Value<T>,
    c: Value<T>,
}

fn main() {}
