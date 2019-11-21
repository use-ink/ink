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

use ink_core_derive::AllocateUsing;
use ink_core::storage::Key;

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
        Key::from([0x0; 32])
    }
}

struct Cell {}

impl ink_core::storage::alloc::AllocateUsing for Cell {
    fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: ink_core::storage::alloc::Allocate,
    {
        alloc.alloc(1);
        Self {}
    }
}

struct Chunk {}

impl ink_core::storage::alloc::AllocateUsing for Chunk {
    fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: ink_core::storage::alloc::Allocate,
    {
        alloc.alloc(100);
        Self {}
    }
}

#[derive(AllocateUsing)]
struct A { a: bool }

#[derive(AllocateUsing)]
struct B { a: i8, b: i16 }

#[derive(AllocateUsing)]
struct C { a: String, b: Vec<u8>, c: [u8; 32] }

#[derive(AllocateUsing)]
struct C<T> { a: Option<T>, b: Vec<T>, c: [T; 32] }

fn main() {}
