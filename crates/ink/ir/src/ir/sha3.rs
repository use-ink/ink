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

/// Computes the KECCAK 256-bit hash for the given input and stores it in output.
pub fn keccak_256(input: &[u8], output: &mut [u8; 32]) {
    use sha3::{
        Digest as _,
        digest::generic_array::GenericArray,
    };
    let mut hasher = sha3::Keccak256::new();
    hasher.update(input);
    hasher.finalize_into(<&mut GenericArray<u8, _>>::from(&mut output[..]));
}

// /// Computes the BLAKE2b-256 bit hash of a string or byte string literal.
// ///
// /// # Note
// ///
// /// This is mainly used for analysis and codegen of the `blake2x256!` macro.
// #[derive(Debug)]
// pub struct Blake2x256Macro {
//     hash: [u8; 32],
//     input: syn::Lit,
// }

// impl Blake2x256Macro {
//     /// Returns the underlying selector.
//     pub fn hash(&self) -> [u8; 32] {
//         self.hash
//     }
//
//     /// Returns the literal input of the BLAKE-2b hash.
//     pub fn input(&self) -> &syn::Lit {
//         &self.input
//     }
// }
//
// impl TryFrom<TokenStream2> for Blake2x256Macro {
//     type Error = syn::Error;
//
//     fn try_from(input: TokenStream2) -> Result<Self, Self::Error> {
//         let input_span = input.span();
//         let lit = syn::parse2::<syn::Lit>(input).map_err(|error| {
//             format_err!(
//                 input_span,
//                 "expected string or byte string literal as input: {}",
//                 error
//             )
//         })?;
//         let input_bytes = match lit {
//             syn::Lit::Str(ref lit_str) => lit_str.value().into_bytes(),
//             syn::Lit::ByteStr(ref byte_str) => byte_str.value(),
//             invalid => {
//                 return Err(format_err!(
//                     invalid.span(),
//                     "expected string or byte string literal as input. found {:?}",
//                     invalid,
//                 ))
//             }
//         };
//         let mut output = [0u8; 32];
//         blake2b_256(&input_bytes, &mut output);
//         Ok(Self {
//             hash: output,
//             input: lit,
//         })
//     }
// }
