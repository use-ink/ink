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

pub trait IsResult {
    fn is_result() -> bool;
}

impl<T> IsResult for T {
    default fn is_result() -> bool {
        false
    }
}

impl<T, E> IsResult for ::core::result::Result<T, E> {
    default fn is_result() -> bool {
        true
    }
}

/// Returns `true` if the given type is a `Result` type.
macro_rules! is_result_type {
    ( $T:ty $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::result_info::IsResult as _;

        <$T as $crate::result_info::IsResult>::is_result()
    }};
}

pub trait IsError {
    fn is_err(&self) -> bool;
}

impl<T> IsError for T {
    default fn is_err(&self) -> bool {
        false
    }
}

impl<T, E> IsError for ::core::result::Result<T, E> {
    default fn is_err(&self) -> bool {
        self.is_err()
    }
}

/// Evaluates to `true` if the given expression is a `Result::Err(_)`.
///
/// # Note
///
/// This given expression is not required to be of type `Result`.
macro_rules! is_result_err {
    ( $e:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::result_info::IsError as _;
        $crate::result_info::IsError::is_err(&$e)
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
    fn is_result_type_works_for_generic() {
        type MyResult = Result<(), ()>;

        fn execute_message<Output, F>(f: F) -> Output
        where
            F: FnOnce() -> Output,
        {
            assert!(is_result_type!(Output));
            f()
        }

        assert!(execute_message(|| -> MyResult { Ok(()) }).is_ok());
    }

    #[test]
    fn is_result_err_works_for_generic() {
        type MyResult = Result<(), ()>;

        fn execute_message<Output, F>(f: F) -> Output
        where
            F: FnOnce() -> Output,
        {
            let result = f();
            assert!(is_result_err!(result));
            result
        }

        assert!(execute_message(|| -> MyResult { Err(()) }).is_err());
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
