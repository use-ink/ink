// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use super::{
    hasher::Hasher,
    Accumulator,
};
use core::marker::PhantomData;

/// Type indicating that no accumulator is in use.
///
/// # Note
///
/// This means that a hash builder with this type as accumulator cannot
/// build hashes for instances based on their SCALE encoding.
#[derive(Debug)]
pub enum NoAccumulator {}

/// Generic hash builder to construct hashes given a builder strategy.
///
/// - `H` defines the crytographic hash to be conducted.
/// - `S` defines the strategy.
///   Currently only accumulating strategy is supported but in the future there
///   might be support for incremental hash builders.
///
/// # Note
///
/// This is just a base type and should not be used directly.
/// Use one of the following concrete hash builders instead:
///
/// - [`Sha2x256`](`crate::hash::Sha2x256`)
/// - [`Keccak256`](`crate::hash::Keccak256`)
/// - [`Blake2x256`](`crate::hash::Blake2x256`)
/// - [`Blake2x128`](`crate::hash::Blake2x128`)
#[derive(Debug)]
pub struct HashBuilder<H, S = NoAccumulator> {
    /// The strategy used to build up the hash.
    strategy: S,
    /// The underlying cryptographic hasher.
    hasher: PhantomData<fn() -> H>,
}

impl<H, S> scale::Output for HashBuilder<H, S>
where
    S: Accumulator,
{
    fn write(&mut self, bytes: &[u8]) {
        <S as Accumulator>::write(&mut self.strategy, bytes)
    }
}

impl<H, S> From<S> for HashBuilder<H, S>
where
    S: Accumulator,
{
    fn from(accumulator: S) -> Self {
        Self {
            hasher: Default::default(),
            strategy: accumulator,
        }
    }
}

impl<H, S> HashBuilder<H, S>
where
    H: Hasher,
{
    /// Conducts the hash for the given bytes.
    ///
    /// Puts the resulting hash into the provided output buffer.
    ///
    /// # Note
    ///
    /// Prefer the simpler [`hash_bytes`](`HashBuilder::hash_bytes`)
    /// if you do _not_ need full control over the `output` buffer.
    pub fn hash_bytes_using(input: &[u8], output: &mut <H as Hasher>::Output)
    where
        H: Hasher,
    {
        <H as Hasher>::finalize_immediate(input, output)
    }

    /// Returns the hash for the given bytes.
    ///
    /// # Note
    ///
    /// Use [`hash_bytes_using`](`HashBuilder::hash_bytes_using`)
    /// if you need full control over the `output` buffer.
    pub fn hash_bytes(input: &[u8]) -> <H as Hasher>::Output
    where
        H: Hasher,
    {
        let mut output = <<H as Hasher>::Output as Default>::default();
        Self::hash_bytes_using(input, &mut output);
        output
    }
}

pub trait Finalize<H>
where
    H: Hasher,
{
    fn finalize_using(&mut self, output: &mut <H as Hasher>::Output);
    fn finalize(&mut self) -> <H as Hasher>::Output;
}

impl<H, S> Finalize<H> for HashBuilder<H, S>
where
    H: Hasher,
    S: Accumulator,
{
    fn finalize_using(&mut self, output: &mut <H as Hasher>::Output) {
        let output = <H as Hasher>::finalize_immediate(self.strategy.as_slice(), output);
        self.strategy.reset();
        output
    }

    fn finalize(&mut self) -> <H as Hasher>::Output {
        let mut output = <<H as Hasher>::Output as Default>::default();
        Self::finalize_using(self, &mut output);
        output
    }
}

impl<H, S> HashBuilder<H, S>
where
    H: Hasher,
    S: Accumulator,
{
    /// Conducts the hash for the encoded input.
    ///
    /// Puts the resulting hash into the provided output buffer.
    ///
    /// # Note
    ///
    /// Prefer the simpler [`hash_encoded`](`HashBuilder::hash_encoded`)
    /// if you do _not_ need full control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// In the following examples the `EXPECTED` buffer is defined as:
    ///
    /// ```
    /// const EXPECTED: [u8; 32] = [
    ///     243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248,  29, 136, 145, 115,
    ///     186, 134, 14, 175, 178, 99, 183,  21,   4, 94,  92,  69, 199, 207, 241, 179,
    /// ];
    /// ```
    ///
    /// ## 1. Using a new `Vec` as accumulating buffer.
    ///
    /// This is the simplest way to call this API but does not avoid heap
    /// allocations.
    ///
    /// ```
    /// # use ink_core::hash::Sha2x256;
    /// # use ink_prelude::vec::Vec;
    /// # const EXPECTED: [u8; 32] = [
    /// #   243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248,  29, 136, 145, 115,
    /// #   186, 134, 14, 175, 178, 99, 183,  21,   4, 94,  92,  69, 199, 207, 241, 179,
    /// # ];
    /// let hashable = (42, "foo", true); // Implements `core::hash::Hash`
    /// let mut output = [0x00_u8; 32]; // 256-bit buffer
    /// let mut hasher = Sha2x256::from(Vec::new());
    /// hasher.hash_encoded_using(&hashable, &mut output);
    /// assert_eq!(output, EXPECTED);
    /// ```
    ///
    /// ## 2. Using an existing `Vec` as accumulating buffer.
    ///
    /// This API is preferred if the call site already has an allocated buffer
    /// that it can reuse. This will reset the existing buffer and might grow it.
    ///
    /// ```
    /// # use ink_core::hash::Sha2x256;
    /// # use ink_prelude::vec::Vec;
    /// # const EXPECTED: [u8; 32] = [
    /// #   243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248,  29, 136, 145, 115,
    /// #   186, 134, 14, 175, 178, 99, 183,  21,   4, 94,  92,  69, 199, 207, 241, 179,
    /// # ];
    /// let hashable = (42, "foo", true); // Implements `core::hash::Hash`
    /// let mut output = [0x00_u8; 32]; // 256-bit buffer
    /// let mut accumulator = Vec::with_capacity(32);
    /// let mut hasher = Sha2x256::from(&mut accumulator);
    /// hasher.hash_encoded_using(&hashable, &mut output);
    /// assert_eq!(output, EXPECTED);
    /// ```
    ///
    /// ## 3. Using a wrapped static buffer as accumulating buffer.
    ///
    /// This API avoids heap allocation completely but might panic in cases
    /// where the static buffer is too small.
    ///
    /// ```
    /// # use ink_core::hash::{Sha2x256, Wrap};
    /// # use ink_prelude::vec::Vec;
    /// # const EXPECTED: [u8; 32] = [
    /// #   243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248,  29, 136, 145, 115,
    /// #   186, 134, 14, 175, 178, 99, 183,  21,   4, 94,  92,  69, 199, 207, 241, 179,
    /// # ];
    /// let hashable = (42, "foo", true); // Implements `core::hash::Hash`
    /// let mut output = [0x00_u8; 32]; // 256-bit buffer
    /// let mut accumulator = [0x00_u8; 64];
    /// let mut hasher = Sha2x256::from(Wrap::from(accumulator.as_mut()));
    /// hasher.hash_encoded_using(&hashable, &mut output);
    /// assert_eq!(output, EXPECTED);
    /// ```
    pub fn hash_encoded_using<T>(&mut self, input: &T, output: &mut <H as Hasher>::Output)
    where
        H: Hasher,
        T: scale::Encode,
    {
        <T as scale::Encode>::encode_to(&input, self);
        self.finalize_using(output)
    }

    /// Returns the hash for the encoded input.
    ///
    /// # Note
    ///
    /// Use [`hash_encoded_using`](`HashBuilder::hash_encoded_using`)
    /// if you need full control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// In the following examples the `EXPECTED` buffer is defined as:
    ///
    /// ```
    /// const EXPECTED: [u8; 32] = [
    ///     243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248,  29, 136, 145, 115,
    ///     186, 134, 14, 175, 178, 99, 183,  21,   4, 94,  92,  69, 199, 207, 241, 179,
    /// ];
    /// ```
    ///
    /// ## 1. Using a new `Vec` as accumulating buffer.
    ///
    /// This is the simplest way to call this API but does not avoid heap
    /// allocations.
    ///
    /// ```
    /// # use ink_core::hash::Sha2x256;
    /// # use ink_prelude::vec::Vec;
    /// # const EXPECTED: [u8; 32] = [
    /// #   243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248,  29, 136, 145, 115,
    /// #   186, 134, 14, 175, 178, 99, 183,  21,   4, 94,  92,  69, 199, 207, 241, 179,
    /// # ];
    /// let hashable = (42, "foo", true); // Implements `core::hash::Hash`
    /// let mut hasher = Sha2x256::from(Vec::new());
    /// assert_eq!(hasher.hash_encoded(&hashable), EXPECTED);
    /// ```
    ///
    /// ## 2. Using an existing `Vec` as accumulating buffer.
    ///
    /// This API is preferred if the call site already has an allocated buffer
    /// that it can reuse. This will reset the existing buffer and might grow it.
    ///
    /// ```
    /// # use ink_core::hash::Sha2x256;
    /// # use ink_prelude::vec::Vec;
    /// # const EXPECTED: [u8; 32] = [
    /// #   243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248,  29, 136, 145, 115,
    /// #   186, 134, 14, 175, 178, 99, 183,  21,   4, 94,  92,  69, 199, 207, 241, 179,
    /// # ];
    /// let hashable = (42, "foo", true); // Implements `core::hash::Hash`
    /// let mut accumulator = Vec::with_capacity(32);
    /// let mut hasher = Sha2x256::from(&mut accumulator);
    /// assert_eq!(hasher.hash_encoded(&hashable), EXPECTED);
    /// ```
    ///
    /// ## 3. Using a wrapped static buffer as accumulating buffer.
    ///
    /// This API avoids heap allocation completely but might panic in cases
    /// where the static buffer is too small.
    ///
    /// ```
    /// # use ink_core::hash::{Sha2x256, Wrap};
    /// # use ink_prelude::vec::Vec;
    /// # const EXPECTED: [u8; 32] = [
    /// #   243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248,  29, 136, 145, 115,
    /// #   186, 134, 14, 175, 178, 99, 183,  21,   4, 94,  92,  69, 199, 207, 241, 179,
    /// # ];
    /// let hashable = (42, "foo", true); // Implements `core::hash::Hash`
    /// let mut accumulator = [0x00_u8; 64];
    /// let mut hasher = Sha2x256::from(Wrap::from(accumulator.as_mut()));
    /// assert_eq!(hasher.hash_encoded(&hashable), EXPECTED);
    /// ```
    pub fn hash_encoded<T>(&mut self, input: &T) -> <H as Hasher>::Output
    where
        H: Hasher,
        T: scale::Encode,
    {
        <T as scale::Encode>::encode_to(&input, self);
        self.finalize()
    }
}
