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

use crate::traits::AtomicGuard;

pub struct IsAtomic<T> {
    marker: core::marker::PhantomData<fn() -> T>,
}

impl<T: AtomicGuard<true>> IsAtomic<T> {
    #[allow(dead_code)]
    pub const VALUE: bool = true;
}

pub trait IsAtomicFallback {
    const VALUE: bool = false;
}
impl<T> IsAtomicFallback for IsAtomic<T> {}

/// Returns `true` if the given type is atomic.
#[macro_export]
#[doc(hidden)]
macro_rules! is_atomic {
    ( $T:ty $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::is_atomic::IsAtomicFallback as _;

        $crate::is_atomic::IsAtomic::<$T>::VALUE
    }};
}

#[cfg(test)]
mod tests {
    use crate::{
        StorageMapping,
        StorageValue,
    };

    #[test]
    fn is_atomic_works() {
        assert!(is_atomic!(u8));
        assert!(is_atomic!(u16));
        assert!(is_atomic!(u32));
        assert!(is_atomic!(u64));
        assert!(is_atomic!(u128));
        assert!(is_atomic!(i8));
        assert!(is_atomic!(i16));
        assert!(is_atomic!(i32));
        assert!(is_atomic!(i64));
        assert!(is_atomic!(i128));

        assert!(is_atomic!(String));

        assert!(is_atomic!(Option<i32>));
        assert!(is_atomic!(Option<u32>));

        assert!(is_atomic!((u32, u32)));
        assert!(is_atomic!((String, ())));
        assert!(is_atomic!(((), ())));
        assert!(is_atomic!(((), String)));
        assert!(is_atomic!((u32, String)));
        assert!(is_atomic!((u32, u32, u32)));

        assert!(is_atomic!(Result<(), ()>));
        assert!(is_atomic!(Result<i32, u32>));
        assert!(is_atomic!(Result<(), String>));
        assert!(is_atomic!(Result<String, ()>));

        // Check that type aliases work, too.
        type MyResult = Result<(), ()>;
        assert!(is_atomic!(MyResult));

        assert!(!is_atomic!(StorageMapping<u32, u32>));
        assert!(!is_atomic!(StorageValue<u32>));
    }
}
