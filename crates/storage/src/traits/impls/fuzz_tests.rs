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

//! Fuzz tests for some storage primitives.

#[cfg(all(test, feature = "std", feature = "ink-fuzz-tests"))]
use quickcheck::TestResult;

#[cfg(all(test, feature = "std", feature = "ink-fuzz-tests"))]
use std::convert::AsMut;

///  Receives a slice, returns an array.
fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

/// Tests if a fuzzed `[i32; 32]` array results in the same object when
/// pushed/pulled from storage (for `spread` and `packed`).
#[quickcheck]
fn fuzz_pull_push_pull_array(x: Vec<i32>) -> TestResult {
    // We want to have only vectors of length 32 fuzzed in here.
    // The reason is that quickcheck does not directly support
    // Array's as a parameter to be fuzzed. So we use this
    // workaround of asking for a Vec with length 32 and convert
    // it to an array with 32 elements subsequently.
    //
    // The guided fuzzing will notice that every Vec of greater/smaller
    // length is always discarded and aim to input vectors of length 32.
    if x.len() != 32 {
        return TestResult::discard()
    }

    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let key = ink_primitives::Key::from([0x42; 32]);
        let key2 = ink_primitives::Key::from([0x77; 32]);

        let arr: [i32; 32] = clone_into_array(&x[0..32]);
        crate::traits::push_spread_root(&arr, &key);

        let y: [i32; 32] = crate::traits::pull_spread_root(&key);
        assert_eq!(arr, y);

        crate::traits::push_packed_root(&arr, &key2);
        let z: [i32; 32] = crate::traits::pull_packed_root(&key2);
        assert_eq!(arr, z);

        Ok(())
    })
    .unwrap();
    TestResult::from_bool(true)
}

/// Tests if a fuzzed `String` results in the same object when pushed/pulled
/// from storage (for `spread` and `packed`).
#[cfg(feature = "ink-fuzz-tests")]
#[quickcheck]
fn fuzz_pull_push_pull_string(x: String) {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let key = ink_primitives::Key::from([0x42; 32]);
        let key2 = ink_primitives::Key::from([0x77; 32]);

        crate::traits::push_spread_root(&x, &key);
        let y: String = crate::traits::pull_spread_root(&key);
        assert_eq!(x, y);

        crate::traits::push_packed_root(&x, &key2);
        let z: String = crate::traits::pull_packed_root(&key2);
        assert_eq!(x, z);
        Ok(())
    })
    .unwrap()
}
