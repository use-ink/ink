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

use ink_core_derive::Flush;

struct Cell {
    // We use this for testing if the Flush implementation is somewhat correct.
    count_flushed: usize,
}

impl ink_core::storage::Flush for Cell {
    fn flush(&mut self) {
        self.count_flushed += 1;
    }
}

struct Chunk {
    // We use this for testing if the Flush implementation is somewhat correct.
    count_flushed: usize,
}

impl ink_core::storage::Flush for Chunk {
    fn flush(&mut self) {
        self.count_flushed += 1;
    }
}

struct StorageVec<T> {
    // We use this for testing if the Flush implementation is somewhat correct.
    count_flushed: usize,
    // The underlying elements.
    //
    // Flush is propagated down to them.
    elems: Vec<T>,
}

impl<T> ink_core::storage::Flush for StorageVec<T>
where
    T: ink_core::storage::Flush,
{
    fn flush(&mut self) {
        for elem in &mut self.elems {
            elem.flush();
        }
    }
}

#[derive(Flush)]
enum Empty {}

#[derive(Flush)]
enum CStyle {
    A, B, C,
}

#[derive(Flush)]
enum TupleStruct {
    A(Cell),
    B(Cell, Chunk),
    C(Cell, Chunk, StorageVec<Cell>),
}

#[derive(Flush)]
enum Struct {
    A { a: bool },
    B { a: i8, b: i16 },
    C { a: String, b: Vec<u8>, c: [u8; 32] },
}

#[derive(Flush)]
enum Mixed {
    A,
    B(String, Vec<u8>, [u8; 32]),
    C { a: String, b: Vec<u8>, c: [u8; 32] },
}

fn main() {}
