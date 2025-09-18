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

pub struct AsOption<'lt, T>(pub &'lt T);

impl<'lt, T> AsOption<'lt, ::core::option::Option<T>> {
    #[inline]
    // We need to allow for dead code at this point because
    // the Rust compiler thinks this function is unused even
    // though it acts as the specialized case for detection.
    #[allow(dead_code)]
    pub fn value(&self) -> Option<&'lt T> {
        self.0.as_ref()
    }
}

impl<'lt, T> AsOption<'lt, &'lt ::core::option::Option<T>> {
    #[inline]
    pub fn value(&self) -> Option<&'lt T> {
        self.0.as_ref()
    }
}

pub trait AsOptionFallback<'lt, T> {
    fn value(&self) -> Option<&'lt T>;
}
impl<'lt, T> AsOptionFallback<'lt, T> for AsOption<'lt, T> {
    #[inline]
    fn value(&self) -> Option<&'lt T> {
        Some(self.0)
    }
}

/// Evaluates to `None` if the given expression is a `Option::None`.
///
/// # Note
///
/// This given expression is not required to be of type `Option`.
#[macro_export]
#[doc(hidden)]
macro_rules! as_option {
    ( $e:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::option_info::AsOptionFallback as _;
        $crate::option_info::AsOption(&$e).value()
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn as_option_works() {
        assert_eq!(Some(&true), as_option!(true));
        assert_eq!(Some(&42), as_option!(42));
        assert_eq!(Some(&"Hello, World!"), as_option!("Hello, World!"));

        assert_eq!(Some(&()), as_option!(Some(())));
        assert_eq!(Some(&5), as_option!(Some(5)));
        assert_eq!(Some(&true), as_option!(Some(true)));
        assert_eq!(Some(&true), as_option!(&Some(true)));

        assert_eq!(None, as_option!(Option::<u32>::None));
        assert_eq!(None, as_option!(Option::<bool>::None));
        assert_eq!(None, as_option!(&Option::<bool>::None));
    }
}
