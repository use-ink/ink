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

mod private {
    /// Used to prevent external direct usage of `HexLiteral::hex_impl_`.
    pub struct Sealed;
}

/// Used to convert literal values into their hex representations for code
/// generation.
pub trait HexLiteral {
    /// Shared implementation details.
    ///
    /// # Note
    ///
    /// Users shall not use this trait method directly hence it is hidden.
    #[doc(hidden)]
    fn hex_impl_(
        self,
        fmt: ::core::fmt::Arguments,
        sealed: private::Sealed,
    ) -> syn::LitInt;

    /// Converts the given value into a hex represented literal with type
    /// suffix.
    fn hex_suffixed(self) -> syn::LitInt;

    /// Converts the given value into a hex represented literal without type
    /// suffix.
    fn hex_unsuffixed(self) -> syn::LitInt;

    /// Converts the given value into a hex represented literal with type
    /// suffix.
    ///
    /// The resulting hex encoded literal is padded with zeros.
    fn hex_padded_suffixed(self) -> syn::LitInt;

    /// Converts the given value into a hex represented literal without type
    /// suffix.
    ///
    /// The resulting hex encoded literal is padded with zeros.
    fn hex_padded_unsuffixed(self) -> syn::LitInt;
}

macro_rules! generate_hex_literal_impls {
    ( $( ($ty:ty, $name:literal, $fmt_suffixed:literal, $fmt_unsuffixed:literal) ),* $(,)? ) => {
        $(
            impl HexLiteral for $ty {
                fn hex_impl_(self, fmt: ::core::fmt::Arguments, _sealed: private::Sealed) -> syn::LitInt {
                    ::syn::LitInt::new(
                        &::std::format!("{}", fmt),
                        proc_macro2::Span::call_site()
                    )
                }

                fn hex_suffixed(self) -> syn::LitInt {
                    self.hex_impl_(::core::format_args!("0x{:X}_{}", self, $name), private::Sealed)
                }

                fn hex_unsuffixed(self) -> syn::LitInt {
                    self.hex_impl_(::core::format_args!("0x{:X}", self), private::Sealed)
                }

                fn hex_padded_suffixed(self) -> syn::LitInt {
                    self.hex_impl_(::core::format_args!($fmt_suffixed, self, $name), private::Sealed)
                }

                fn hex_padded_unsuffixed(self) -> syn::LitInt {
                    self.hex_impl_(::core::format_args!($fmt_unsuffixed, self), private::Sealed)
                }
            }
        )*
    };
}
#[rustfmt::skip]
generate_hex_literal_impls!(
    (::core::primitive::i8 ,    "i8",  "0x{:02X}_{}",  "0x{:02X}"),
    (::core::primitive::u8 ,    "u8",  "0x{:02X}_{}",  "0x{:02X}"),
    (::core::primitive::i16,   "i16",  "0x{:04X}_{}",  "0x{:04X}"),
    (::core::primitive::u16,   "u16",  "0x{:04X}_{}",  "0x{:04X}"),
    (::core::primitive::i32,   "i32",  "0x{:08X}_{}",  "0x{:08X}"),
    (::core::primitive::u32,   "u32",  "0x{:08X}_{}",  "0x{:08X}"),
    (::core::primitive::i64,   "i64", "0x{:016X}_{}", "0x{:016X}"),
    (::core::primitive::u64,   "u64", "0x{:016X}_{}", "0x{:016X}"),
    (::core::primitive::i128, "i128", "0x{:032X}_{}", "0x{:032X}"),
    (::core::primitive::u128, "u128", "0x{:032X}_{}", "0x{:032X}"),
);
#[cfg(target_pointer_width = "16")]
generate_hex_literal_impls!(
    (::core::primitive::usize, "usize", "0x{:04X}_{}", "0x{:04X}"),
    (::core::primitive::isize, "isize", "0x{:04X}_{}", "0x{:04X}"),
);
#[cfg(target_pointer_width = "32")]
generate_hex_literal_impls!(
    (::core::primitive::usize, "usize", "0x{:08X}_{}", "0x{:08X}"),
    (::core::primitive::isize, "isize", "0x{:08X}_{}", "0x{:08X}"),
);
#[cfg(target_pointer_width = "64")]
generate_hex_literal_impls!(
    (::core::primitive::usize, "usize", "0x{:16X}_{}", "0x{:16X}"),
    (::core::primitive::isize, "isize", "0x{:16X}_{}", "0x{:16X}"),
);

#[cfg(test)]
mod tests {
    use super::HexLiteral as _;
    use quote::quote;

    fn assert_quote(given: syn::LitInt, expected: &str) {
        assert_eq!(quote!(#given).to_string(), expected);
    }

    #[test]
    fn simple() {
        let value = 0x42_i32;
        assert_eq!(quote!(#value).to_string(), "66i32");
        assert_quote(value.hex_suffixed(), "0x42_i32");
        assert_quote(value.hex_unsuffixed(), "0x42");
        assert_quote(value.hex_padded_suffixed(), "0x00000042_i32");
        assert_quote(value.hex_padded_unsuffixed(), "0x00000042");
    }

    #[test]
    fn code_cafe() {
        let value = 0xC0DE_CAFE_u32;
        assert_quote(value.hex_suffixed(), "0xC0DECAFE_u32");
        assert_quote(value.hex_padded_suffixed(), "0xC0DECAFE_u32");
        assert_quote(value.hex_unsuffixed(), "0xC0DECAFE");
        assert_quote(value.hex_padded_unsuffixed(), "0xC0DECAFE");
    }
}
