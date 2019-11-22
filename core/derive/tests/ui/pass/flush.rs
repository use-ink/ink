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

use ink_core_derive::Flush;
use utils::*;

#[derive(Debug, Default, PartialEq, Eq)]
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
        self.count_flushed += 1;
        for elem in &mut self.elems {
            elem.flush();
        }
    }
}

#[derive(Flush, Debug, Default, PartialEq, Eq)]
struct UnitStruct;

#[derive(Flush, Debug, Default, PartialEq, Eq)]
struct NewtypeStruct(Cell);

#[derive(Flush, Debug, Default, PartialEq, Eq)]
struct NamedStruct {
    a: Cell,
    b: Chunk,
}

#[derive(Flush, Debug, Default, PartialEq, Eq)]
struct ComplexNamedStruct {
    a: Chunk,
    b: Value<Cell>,
    c: Value<Chunk>,
}

#[derive(Flush, Debug, Default, PartialEq, Eq)]
struct GenericNamedStruct<T> {
    a: Cell,
    b: Chunk,
    c: Value<T>,
    d: StorageVec<Value<T>>,
}

#[derive(Flush, Debug, PartialEq, Eq)]
enum CStyleEnum {
    A,
    B,
    C,
}

impl Default for CStyleEnum {
    fn default() -> Self {
        Self::A
    }
}

#[derive(Flush, Debug, PartialEq, Eq)]
enum TupleStructEnum {
    A(Cell),
    B(Cell, Chunk),
    C(Cell, Chunk, StorageVec<Cell>),
}

impl Default for TupleStructEnum {
    fn default() -> Self {
        Self::C(Cell::default(), Chunk::default(), StorageVec::<Cell>::default())
    }
}

#[derive(Flush, Debug, PartialEq, Eq)]
enum StructEnum {
    A {
        a: Cell,
    },
    B {
        a: Cell,
        b: Chunk,
    },
    C {
        a: Value<Cell>,
        b: StorageVec<Cell>,
        c: StorageVec<Value<Chunk>>,
    },
}

impl Default for StructEnum {
    fn default() -> Self {
        Self::C {
            a: Value::default(),
            b: StorageVec::default(),
            c: StorageVec::default(),
        }
    }
}

#[derive(Flush, Debug, PartialEq, Eq)]
enum MixedEnum {
    A,
    B(Cell, Value<Cell>, StorageVec<Cell>),
    C {
        a: Chunk,
        b: Value<Cell>,
        c: StorageVec<Chunk>,
    },
}

impl Default for MixedEnum {
    fn default() -> Self {
        Self::C {
            a: Chunk::default(),
            b: Value::default(),
            c: StorageVec::<Chunk>::default(),
        }
    }
}

fn test_for<T>(expected: T)
where
    T: Default + ink_core::storage::Flush + PartialEq + Eq + core::fmt::Debug,
{
    let mut input = T::default();
    input.flush();
    assert_eq!(input, expected);
}

fn main() {
    test_for::<UnitStruct>(Default::default());
    test_for::<NewtypeStruct>(NewtypeStruct(Cell { count_flushed: 1 }));
    test_for::<NamedStruct>(NamedStruct {
        a: Cell { count_flushed: 1 },
        b: Chunk { count_flushed: 1 }
    });
    test_for::<ComplexNamedStruct>(ComplexNamedStruct {
        a: Chunk { count_flushed: 1 },
        b: Value { value: Cell { count_flushed: 1 } },
        c: Value { value: Chunk { count_flushed: 1 } },
    });
    test_for::<GenericNamedStruct<Cell>>(GenericNamedStruct::<Cell> {
        a: Cell { count_flushed: 1 },
        b: Chunk { count_flushed: 1 },
        c: Value { value: Cell { count_flushed: 1 } },
        d: StorageVec { count_flushed: 1, elems: vec![] },
    });
    test_for::<CStyleEnum>(Default::default());
    test_for::<TupleStructEnum>(TupleStructEnum::C(
        Cell { count_flushed: 1 },
        Chunk { count_flushed: 1 },
        StorageVec { count_flushed: 1, elems: vec![] }
    ));
    test_for::<StructEnum>(StructEnum::C {
        a: Value { value: Cell { count_flushed: 1 } },
        b: StorageVec { count_flushed: 1, elems: vec![] },
        c: StorageVec { count_flushed: 1, elems: vec![] },
    });
    test_for::<MixedEnum>(MixedEnum::C {
        a: Chunk { count_flushed: 1 },
        b: Value { value: Cell { count_flushed: 1 } },
        c: StorageVec { count_flushed: 1, elems: vec![] },
    })
}
