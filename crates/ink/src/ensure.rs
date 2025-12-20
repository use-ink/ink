// Copyright (C) Use Ink (UK) Ltd.
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

/// Evaluate `$condition:expr` and if not true return `Err($error:expr)`.
///
/// This macro is similar to `frame_support::ensure!` and provides a convenient
/// way to check conditions and return errors in ink! contracts.
///
/// # Example
///
/// # use ink::ensure;
/// # #[derive(Debug, PartialEq, Eq)]
/// # #[ink::error]
/// # pub enum Error {
/// # InsufficientBalance,
/// # }
/// # pub type Result<T> = core::result::Result<T, Error>;
/// #
/// # fn example(balance: u32, amount: u32) -> Result<()> {
/// ensure!(balance >= amount, Error::InsufficientBalance);
/// // ... rest of the function
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! ensure {
    ( $condition:expr, $error:expr $(,)? ) => {{
        if !$condition {
            return ::core::result::Result::Err(::core::convert::Into::into($error));
        }
    }};
}

#[cfg(test)]
mod tests {

    #[derive(Debug, PartialEq, Eq)]
    enum TestError {
        TooSmall,
        TooLarge,
    }
    type TestResult<T> = core::result::Result<T, TestError>;

    #[test]
    fn ensure_works_when_condition_is_true() {
        fn test_function(value: u32) -> TestResult<()> {
            crate::ensure!(value > 0, TestError::TooSmall);
            crate::ensure!(value < 100, TestError::TooLarge);
            Ok(())
        }
        // This should succeed when the conditions are met
        assert_eq!(test_function(50), Ok(()));
    }
    #[test]
    fn ensure_returns_error_when_condition_is_false() {
        fn test_function(value: u32) -> TestResult<()> {
            crate::ensure!(value > 10, TestError::TooSmall);
            Ok(())
        }
        // This should return error when condition fails
        assert_eq!(test_function(5), Err(TestError::TooSmall));
    }
    #[test]
    fn ensure_works_with_trailing_comma() {
        fn test_function(value: u32) -> TestResult<()> {
            crate::ensure!(value > 0, TestError::TooSmall,);
            Ok(())
        }

        assert!(test_function(1).is_ok());
        assert_eq!(test_function(0), Err(TestError::TooSmall));
    }

    #[test]
    fn ensure_works_with_string_error() {
        fn test_function(value: u32) -> Result<(), String> {
            crate::ensure!(value > 0, "Value must be positive".to_string());
            Ok(())
        }

        assert!(test_function(1).is_ok());
        assert_eq!(test_function(0), Err("Value must be positive".to_string()));
    }
}
