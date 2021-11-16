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

pub struct IsResultType<T> {
    marker: core::marker::PhantomData<fn() -> T>,
}

impl<T, E> IsResultType<::core::result::Result<T, E>> {
    // We need to allow for dead code at this point because
    // the Rust compiler thinks this function is unused even
    // though it acts as the specialized case for detection.
    #[allow(dead_code)]
    pub const VALUE: bool = true;
}

pub trait IsResultTypeFallback {
    const VALUE: bool = false;
}
impl<T> IsResultTypeFallback for IsResultType<T> {}

/// Returns `true` if the given type is a `Result` type.
#[macro_export]
#[doc(hidden)]
macro_rules! is_result_type {
    ( $T:ty $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::result_info::IsResultTypeFallback as _;

        $crate::result_info::IsResultType::<$T>::VALUE
    }};
}

pub struct IsResultErr<'lt, T>(pub &'lt T);

impl<T, E> IsResultErr<'_, ::core::result::Result<T, E>> {
    // We need to allow for dead code at this point because
    // the Rust compiler thinks this function is unused even
    // though it acts as the specialized case for detection.
    #[allow(dead_code)]
    pub fn value(&self) -> bool {
        self.0.is_err()
    }
}

pub trait IsResultErrFallback {
    fn value(&self) -> bool {
        false
    }
}
impl<T> IsResultErrFallback for IsResultErr<'_, T> {}

/// Evaluates to `true` if the given expression is a `Result::Err(_)`.
///
/// # Note
///
/// This given expression is not required to be of type `Result`.
#[macro_export]
#[doc(hidden)]
macro_rules! is_result_err {
    ( $e:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::result_info::IsResultErrFallback as _;
        $crate::result_info::IsResultErr(&$e).value()
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn is_result_type_works() {
        assert!(!is_result_type!(bool));
        assert!(!is_result_type!(String));
        assert!(!is_result_type!(Option<i32>));

        assert!(is_result_type!(Result<(), ()>));
        assert!(is_result_type!(Result<i32, u32>));
        assert!(is_result_type!(Result<(), String>));
        assert!(is_result_type!(Result<String, ()>));

        assert!(is_result_type!(Result<Result<(), ()>, ()>));
        assert!(is_result_type!(Result<(), Result<(), ()>>));
        assert!(is_result_type!(Result<Result<(), ()>, Result<(), ()>>));

        // Check that type aliases work, too.
        type MyResult = Result<(), ()>;
        assert!(is_result_type!(MyResult));
    }

    #[test]
    fn is_result_err_works() {
        assert!(!is_result_err!(true));
        assert!(!is_result_err!(42));
        assert!(!is_result_err!("Hello, World!"));

        assert!(!is_result_err!(Ok::<(), ()>(())));
        assert!(!is_result_err!(Ok::<i32, ()>(5)));
        assert!(!is_result_err!(Ok::<bool, String>(true)));

        assert!(is_result_err!(Err::<(), ()>(())));
        assert!(is_result_err!(Err::<(), i32>(5)));
        assert!(is_result_err!(Err::<i32, bool>(false)));
        assert!(is_result_err!(Err::<i32, Result::<i32, String>>(Ok(42))));

        {
            // Check that we do not simply check against `Result` as identifier.
            type Result = Option<()>;
            assert!(!is_result_type!(Result));
        }
    }
}
