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

//! Abstractions for implementing Solidity ABI encoding/decoding for arbitrary Rust types.

pub use ink_primitives::sol::*;

/// Computes the Keccak-256 hash of the given string.
///
/// # Note
///
/// The input can be a const expression.
#[macro_export]
macro_rules! keccak_256 {
    ($input: expr) => {
        const { $crate::codegen::sol::keccak_256($input.as_bytes()) }
    };
}

/// Returns the selector of the equivalent [Solidity custom error][sol-error]
/// for given the name (as a `const` expression) and a tuple type
/// representing the error parameters types.
///
/// # Note
///
/// Each tuple element type must implement [`SolEncode`][crate::SolEncode]
/// and [`SolDecode`][crate::SolDecode].
///
/// [sol-error]: https://soliditylang.org/blog/2021/04/21/custom-errors/
#[macro_export]
macro_rules! sol_error_selector {
    ($name: expr, $params_ty: ty) => {
        const {
            $crate::codegen::sol::selector_bytes($crate::codegen::utils::const_concat!(
                $name,
                <$params_ty as $crate::sol::SolParamsDecode>::SOL_NAME
            ))
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn error_selector_works() {
        // `keccak256("UnitError()")` == `0xe930c64c`
        assert_eq!(
            sol_error_selector!("UnitError", ()),
            [0xe9, 0x30, 0xc6, 0x4c]
        );

        // `keccak256("ErrorWithParams(bool)")` == `0xac3a6266`
        assert_eq!(
            sol_error_selector!("ErrorWithParams", (bool,)),
            [0xac, 0x3a, 0x62, 0x66]
        );
    }
}
