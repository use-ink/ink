// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

use super::BitStash;

/// Conducts repeated insert and remove operations into the stash by iterating
/// over `xs`. Typically the `xs` and `inserts_each` arguments are provided
/// by our fuzzing engine in an iterative manner.
///
/// For each odd `x` in `xs` a number of put operations are executed.
/// For each even `x` it is asserted that the previously inserted elements
/// are in the stash and they are taken out subsequently.
///
/// The reasoning behind this even/odd sequence is to introduce some
/// randomness into when elements are inserted/removed.
///
/// `inserts_each` was chosen as `u8` to keep the number of inserts per `x` in
/// a reasonable range.
fn put_and_take(xs: Vec<i32>, additional_puts_each: u8) {
    let mut stash = BitStash::new();
    let mut previous_even_x = None;
    let mut last_put_indices = Vec::new();

    for x in 0..xs.len() as u32 {
        if x % 2 == 0 {
            // On even numbers we put
            let mut put_index = None;
            for _ in 0..x + additional_puts_each as u32 {
                let index = stash.put();
                assert_eq!(stash.get(index), Some(true));
                last_put_indices.push(index);
                put_index = Some(index);
            }
            if previous_even_x.is_none() && put_index.is_some() {
                previous_even_x = put_index;
            }
        } else if previous_even_x.is_some() {
            // If it's an odd number and we inserted in a previous run we assert
            // that the last insert worked correctly and remove the elements again.
            //
            // It can happen that after one insert run there are many more
            // insert runs (i.e. more susbequent even `x` in `xs`) before we remove
            // the numbers of the last run again. This is intentional, as to include
            // testing if subsequent insert operations have an effect on already
            // inserted items.
            while let Some(index) = last_put_indices.pop() {
                assert_eq!(stash.get(index), Some(true));
                assert_eq!(stash.take(index), Some(true));
                assert_eq!(stash.get(index), Some(false));
            }
            previous_even_x = None;
        }
    }
}

#[quickcheck]
fn fuzz_repeated_puts_and_takes(xs: Vec<i32>, additional_puts_each: u8) {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        put_and_take(xs, additional_puts_each);
        Ok(())
    })
    .unwrap()
}
