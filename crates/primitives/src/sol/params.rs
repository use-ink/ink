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
    SolType as AlloySolType,
    abi::{
        self,
        Encoder,
    },
};
use impl_trait_for_tuples::impl_for_tuples;
use ink_prelude::vec::Vec;

use super::{
    Error,
    SolDecode,
    SolEncode,
    SolTypeDecode,
    SolTypeEncode,
    encodable::{
        Encodable,
        EncodableParams,
    },
};

/// Solidity ABI decode from parameter data (e.g. function, event or error parameters).
///
/// # Note
///
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolParamsDecode: SolDecode + Sized + private::Sealed {
    /// Name of equivalent Solidity ABI type.
    const SOL_NAME: &'static str = <Self as SolDecode>::SOL_NAME;

    /// Solidity ABI decode parameter data into this type.
    fn decode(data: &[u8]) -> Result<Self, Error>;
}

/// Solidity ABI encode as a parameter sequence (e.g. function, event or error
/// parameters).
///
/// # Note
///
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolParamsEncode<'a>: SolEncode<'a> + private::Sealed {
    /// Name of equivalent Solidity ABI type.
    const SOL_NAME: &'static str = <Self as SolEncode<'a>>::SOL_NAME;

    /// Solidity ABI encode the value as a parameter sequence.
    fn encode(&'a self) -> Vec<u8>;
}

// We follow the Rust standard library's convention of implementing traits for tuples up
// to twelve items long.
// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
#[impl_for_tuples(1, 12)]
#[tuple_types_custom_trait_bound(SolDecode)]
impl SolParamsDecode for Tuple {
    fn decode(data: &[u8]) -> Result<Self, Error> {
        abi::decode_params::<
            <<<Self as SolDecode>::SolType as SolTypeDecode>::AlloyType as AlloySolType>::Token<'_>,
        >(data)
            .map_err(Error::from)
            .and_then(<<Self as SolDecode>::SolType as SolTypeDecode>::detokenize)
            .and_then(<Self as SolDecode>::from_sol_type)
    }
}

#[impl_for_tuples(1, 12)]
#[tuple_types_custom_trait_bound(SolEncode<'a>)]
impl<'a> SolParamsEncode<'a> for Tuple {
    fn encode(&'a self) -> Vec<u8> {
        let params = self.to_sol_type();
        let token = <<Self as SolEncode>::SolType as SolTypeEncode>::tokenize(&params);
        let mut encoder = Encoder::with_capacity(token.total_words());
        EncodableParams::encode_params(&token, &mut encoder);
        encoder.into_bytes()
    }
}

// Optimized implementations for unit (i.e. `()`).
impl SolParamsDecode for () {
    fn decode(_: &[u8]) -> Result<Self, Error> {
        // NOTE: Solidity ABI decoding doesn't validate input length.
        Ok(())
    }
}

impl SolParamsEncode<'_> for () {
    fn encode(&self) -> Vec<u8> {
        Vec::new()
    }
}

#[impl_for_tuples(12)]
#[tuple_types_no_default_trait_bound]
impl private::Sealed for Tuple {}

mod private {
    /// Seals implementations of `SolParamsEncode` and `SolParamsDecode`.
    pub trait Sealed {}
}
