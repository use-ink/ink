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

//! Utilities for testing if the storage interaction of an object
//! which is pushed/pulled/cleared to/from storage behaves as it should.

/// Runs `f` using the off-chain testing environment.
#[cfg(test)]
pub fn run_test<F>(f: F)
where
    F: FnOnce(),
{
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        f();
        Ok(())
    })
    .unwrap()
}

/// Creates two tests:
/// (1) Tests if an object which is `push_spread`-ed to storage results in exactly
///     the same object when it is `pull_spread`-ed again. Subsequently the object
///     undergoes the same test for `push_packed` and `pull_packed`.
/// (2) Tests if `clear_spread` removes the object properly from storage.
#[macro_export]
macro_rules! push_pull_works_for_primitive {
    ( $name:ty, [$($value:expr),*] ) => {
        paste::item! {
            #[test]
            #[allow(non_snake_case)]
            fn [<$name _pull_push_works>] () {
                crate::test_utils::run_test(|| {
                    $({
                        let x: $name = $value;
                        let key = ink_primitives::Key::from([0x42; 32]);
                        let key2 = ink_primitives::Key::from([0x77; 32]);
                        crate::traits::push_spread_root(&x, &key);
                        let y: $name = crate::traits::pull_spread_root(&key);
                        assert_eq!(x, y);
                        crate::traits::push_packed_root(&x, &key2);
                        let z: $name = crate::traits::pull_packed_root(&key2);
                        assert_eq!(x, z);
                    })*
                })
            }

            #[test]
            #[should_panic(expected = "storage entry was empty")]
            #[allow(non_snake_case)]
            fn [<$name _clean_works>]() {
                crate::test_utils::run_test(|| {
                    $({
                        let x: $name = $value;
                        let key = ink_primitives::Key::from([0x42; 32]);
                        crate::traits::push_spread_root(&x, &key);
                        // Works since we just populated the storage.
                        let y: $name = crate::traits::pull_spread_root(&key);
                        assert_eq!(x, y);
                        crate::traits::clear_spread_root(&x, &key);
                        // Panics since it loads eagerly from cleared storage.
                        let _: $name = crate::traits::pull_spread_root(&key);
                    })*
                })
            }
        }
    };
}
