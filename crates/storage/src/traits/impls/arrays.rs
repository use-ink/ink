// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use crate::traits::{
    AtomicGuard,
    AutoKey,
    StorageKeyHolder,
    StorageType,
};

impl<T: AtomicGuard<true>, const N: usize> AtomicGuard<true> for [T; N] {}

impl<
        T: AtomicGuard<true> + StorageType<Salt>,
        const N: usize,
        Salt: StorageKeyHolder,
    > StorageType<Salt> for [T; N]
{
    type Type = [T::Type; N];
    type PreferredKey = AutoKey;
}

#[cfg(test)]
mod tests {
    use crate::storage_type_works_for_primitive;

    type Array = [i32; 4];
    storage_type_works_for_primitive!(Array);

    type ArrayTuples = [(i32, i32); 2];
    storage_type_works_for_primitive!(ArrayTuples);
}
