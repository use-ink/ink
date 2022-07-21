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

#[cfg(test)]
mod tests {
    use crate::item_works_for_primitive;
    use ink_env::AccountId;

    item_works_for_primitive!(bool);
    item_works_for_primitive!(String);
    item_works_for_primitive!(AccountId);
    item_works_for_primitive!(i8);
    item_works_for_primitive!(i16);
    item_works_for_primitive!(i32);
    item_works_for_primitive!(i64);
    item_works_for_primitive!(i128);
    item_works_for_primitive!(u8);
    item_works_for_primitive!(u16);
    item_works_for_primitive!(u32);
    item_works_for_primitive!(u64);
    item_works_for_primitive!(u128);

    type OptionU8 = Option<u8>;
    item_works_for_primitive!(OptionU8);

    type ResultU8 = Result<u8, bool>;
    item_works_for_primitive!(ResultU8);

    type BoxU8 = Box<u8>;
    item_works_for_primitive!(BoxU8);

    type BoxOptionU8 = Box<Option<u8>>;
    item_works_for_primitive!(BoxOptionU8);
}
