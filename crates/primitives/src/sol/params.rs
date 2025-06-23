// Copyright (C) ink! contributors.
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

use alloy_sol_types::{
    abi,
    SolType as AlloySolType,
};
use impl_trait_for_tuples::impl_for_tuples;
use ink_prelude::vec::Vec;

use super::{
    SolDecode,
    SolEncode,
    SolTypeDecode,
    SolTypeEncode,
};

/// Solidity ABI decode function parameters.
///
/// # Note
///
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolParamsDecode: Sized + private::Sealed {
    /// Solidity ABI decode function parameters into this type.
    fn decode(data: &[u8]) -> Result<Self, alloy_sol_types::Error>;
}

/// Solidity ABI encode function parameters.
///
/// # Note
///
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolParamsEncode: private::Sealed {
    /// Solidity ABI encode the value as function parameters.
    fn encode(&self) -> Vec<u8>;
}

// We follow the Rust standard library's convention of implementing traits for tuples up
// to twelve items long.
// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
#[impl_for_tuples(12)]
#[tuple_types_custom_trait_bound(SolDecode)]
impl SolParamsDecode for Tuple {
    fn decode(data: &[u8]) -> Result<Self, alloy_sol_types::Error> {
        abi::decode_params::<
            <<<Self as SolDecode>::SolType as SolTypeDecode>::AlloyType as AlloySolType>::Token<'_>,
        >(data)
            .and_then(<<Self as SolDecode>::SolType as SolTypeDecode>::detokenize)
            .map(<Self as SolDecode>::from_sol_type)
    }
}

#[impl_for_tuples(12)]
#[tuple_types_custom_trait_bound(for<'a> SolEncode<'a>)]
impl SolParamsEncode for Tuple {
    fn encode(&self) -> Vec<u8> {
        abi::encode_params(&<<Self as SolEncode>::SolType as SolTypeEncode>::tokenize(
            &self.to_sol_type(),
        ))
    }
}

#[impl_for_tuples(12)]
#[tuple_types_no_default_trait_bound]
impl private::Sealed for Tuple {}

mod private {
    /// Seals implementations of `SolParamsEncode` and `SolParamsDecode`.
    pub trait Sealed {}
}
