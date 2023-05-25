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

pub struct IsOptionType<T> {
    marker: core::marker::PhantomData<fn() -> T>,
}

impl<T> IsOptionType<::core::option::Option<T>> {
    // We need to allow for dead code at this point because
    // the Rust compiler thinks this function is unused even
    // though it acts as the specialized case for detection.
    #[allow(dead_code)]
    pub const VALUE: bool = true;
}

pub trait IsOptionTypeFallback {
    const VALUE: bool = false;
}
impl<T> IsOptionTypeFallback for IsOptionType<T> {}

/// Returns `true` if the given type is a `Result` type.
#[macro_export]
#[doc(hidden)]
macro_rules! is_option_type {
    ( $T:ty $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::option_info::IsOptionTypeFallback as _;

        $crate::option_info::IsOptionType::<$T>::VALUE
    }};
}

pub struct IsOptionNone<'lt, T>(pub &'lt T);

impl<T> IsOptionNone<'_, ::core::option::Option<T>> {
    #[inline]
    // We need to allow for dead code at this point because
    // the Rust compiler thinks this function is unused even
    // though it acts as the specialized case for detection.
    #[allow(dead_code)]
    pub fn value(&self) -> bool {
        self.0.is_none()
    }
}

pub trait IsOptionNoneFallback {
    #[inline]
    fn value(&self) -> bool {
        false
    }
}
impl<T> IsOptionNoneFallback for IsOptionNone<'_, T> {}

/// Evaluates to `true` if the given expression is a `Option::None`.
///
/// # Note
///
/// This given expression is not required to be of type `Result`.
#[macro_export]
#[doc(hidden)]
macro_rules! is_option_none {
    ( $e:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::option_info::IsOptionNoneFallback as _;
        $crate::option_info::IsOptionNone(&$e).value()
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn is_option_type_works() {
        assert!(!is_option_type!(bool));
        assert!(!is_option_type!(String));
        assert!(!is_option_type!(Result<i32, u32>));

        assert!(is_option_type!(Option<()>));
        assert!(is_option_type!(Option<i32>));
        assert!(is_option_type!(Option<String>));
        assert!(is_option_type!(Option<&str>));

        assert!(is_option_type!(Option<Option<()>>));
        assert!(is_option_type!(Option<(Option<()>, Option<()>)>));

        // Check that type aliases work, too.
        type MyOption = Option<()>;
        assert!(is_option_type!(MyOption));
    }

    #[test]
    fn is_option_none_works() {
        assert!(!is_option_none!(true));
        assert!(!is_option_none!(42));
        assert!(!is_option_none!("Hello, World!"));

        assert!(!is_option_none!(Some(())));
        assert!(!is_option_none!(Some(5)));
        assert!(!is_option_none!(Some(true)));

        assert!(is_option_none!(Option::<u32>::None));
        {
            // Check that we do not simply check against `Option` as identifier.
            type Option = Result<(), ()>;
            assert!(!is_option_type!(Option));
        }
    }
}
