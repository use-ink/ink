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

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Cell {
    // We use this for testing if the Flush implementation is somewhat correct.
    pub count_flushed: usize,
}

impl ink_core::storage::Flush for Cell {
    fn flush(&mut self) {
        self.count_flushed += 1;
    }
}

impl ink_core::storage::alloc::AllocateUsing for Cell {
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: ink_core::storage::alloc::Allocate,
    {
        alloc.alloc(1);
        Self { count_flushed: 0 }
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Chunk {
    // We use this for testing if the Flush implementation is somewhat correct.
    pub count_flushed: usize,
}

impl ink_core::storage::Flush for Chunk {
    fn flush(&mut self) {
        self.count_flushed += 1;
    }
}

impl ink_core::storage::alloc::AllocateUsing for Chunk {
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: ink_core::storage::alloc::Allocate,
    {
        alloc.alloc(100);
        Self { count_flushed: 0 }
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Value<T> {
    pub value: T,
}

impl<T> ink_core::storage::Flush for Value<T>
where
    T: ink_core::storage::Flush,
{
    fn flush(&mut self) {
        self.value.flush()
    }
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
