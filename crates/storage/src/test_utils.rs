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

/// Creates test to verify that the type for primitives is atomic and the same.
#[macro_export]
macro_rules! storage_type_works_for_primitive {
    ( $ty:ty ) => {
        paste::item! {
            #[test]
            #[allow(non_snake_case)]
            fn [<$ty _storage_type_works>] () {
                $crate::test_utils::run_test(|| {
                    assert_eq!(
                        ::core::any::TypeId::of::<$ty>(),
                        ::core::any::TypeId::of::<<$ty as $crate::traits::StorageType<$crate::traits::ManualKey<0>>>::Type>()
                    );
                    assert!($crate::is_atomic!($ty));
                })
            }
        }
    };
}
