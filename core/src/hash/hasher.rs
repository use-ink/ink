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

use core::{
    hash::{
        BuildHasherDefault,
        Hasher,
    },
    marker::PhantomData,
};
use ink_prelude::vec::Vec;

mod markers {
    pub trait Sealed {}

    pub enum Sha2x256Marker {}
    pub enum Keccakx256Marker {}
    pub enum Blake2x256Marker {}
    pub enum Blake2x128Marker {}
    pub enum TwoxMarker {}

    impl Sealed for Sha2x256Marker {}
    impl Sealed for Keccakx256Marker {}
    impl Sealed for Blake2x256Marker {}
    impl Sealed for Blake2x128Marker {}
    impl Sealed for TwoxMarker {}
}

/// Generic cryptographic hasher.
pub struct CryptoHasher<T, I>
where
    T: markers::Sealed,
{
    /// The input buffer.
    ///
    /// This bytes buffer is used to accumulate a hashing state.
    input: I,
    /// Tricks compiler into thinking that `Self` uses `T`.
    marker: PhantomData<T>,
}

/// SHA2 256-bit hasher.
pub type Sha2x256Hasher<I> = CryptoHasher<markers::Sha2x256Marker, I>;
/// KECCAK 256-bit hasher.
pub type Keccakx256Hasher<I> = CryptoHasher<markers::Keccakx256Marker, I>;
/// BLAKE2 256-bit hasher.
pub type Blake2x256Hasher<I> = CryptoHasher<markers::Blake2x256Marker, I>;
/// BLAKE2 128-bit hasher.
pub type Blake2x128Hasher<I> = CryptoHasher<markers::Blake2x128Marker, I>;
/// TWOX 256, 128 or 64-bit hasher.
pub type TwoxHasher<I> = CryptoHasher<markers::TwoxMarker, I>;

/// Types that qualify as input buffer.
///
/// # Examples
///
/// Rust's `Vec<u8>` types and an exclusive reference to it (`&mut Vec<u8>`)
/// qualify as such. Users may implement this trait for other types. E.g. it
/// could be useful to have a `SmallVec` or a static buffer implementation for
/// this trait.
pub trait InputBuffer {
    fn reset(&mut self);
    fn write(&mut self, bytes: &[u8]);
    fn as_slice(&self) -> &[u8];
}

impl InputBuffer for Vec<u8> {
    fn reset(&mut self) {
        <Vec<_>>::clear(self)
    }

    fn write(&mut self, bytes: &[u8]) {
        <Vec<_>>::extend_from_slice(self, bytes)
    }

    fn as_slice(&self) -> &[u8] {
        <Vec<_>>::as_slice(self)
    }
}

impl<'a> InputBuffer for &'a mut Vec<u8> {
    fn reset(&mut self) {
        <Vec<_>>::clear(self)
    }

    fn write(&mut self, bytes: &[u8]) {
        <Vec<_>>::extend_from_slice(self, bytes)
    }

    fn as_slice(&self) -> &[u8] {
        <Vec<_>>::as_slice(self)
    }
}

impl<T> Default for CryptoHasher<T, Vec<u8>>
where
    T: markers::Sealed,
{
    fn default() -> Self {
        Self::from(Vec::new())
    }
}

impl<T, I> From<I> for CryptoHasher<T, I>
where
    T: markers::Sealed,
    I: InputBuffer,
{
    fn from(mut input: I) -> Self {
        <I as InputBuffer>::reset(&mut input);
        Self {
            input,
            marker: PhantomData,
        }
    }
}

pub trait FinishInto<Output> {
    fn finish_into(&self, output: &mut Output);
}

pub trait Finish<Output> {
    fn finish(&self) -> Output;
}

impl<T, I, O> Finish<O> for CryptoHasher<T, I>
where
    Self: FinishInto<O>,
    T: markers::Sealed,
    I: InputBuffer,
    O: AsMut<[u8]> + Default,
{
    fn finish(&self) -> O {
        let mut output = <O as Default>::default();
        <Self as FinishInto<O>>::finish_into(self, &mut output);
        output
    }
}

impl<I> FinishInto<[u8; 32]> for Sha2x256Hasher<I>
where
    I: InputBuffer,
{
    fn finish_into(&self, output: &mut [u8; 32]) {
        super::sha2_256_raw_into(self.input.as_slice(), output)
    }
}

impl<I> FinishInto<[u8; 32]> for Keccakx256Hasher<I>
where
    I: InputBuffer,
{
    fn finish_into(&self, output: &mut [u8; 32]) {
        super::keccak_256_raw_into(self.input.as_slice(), output)
    }
}

impl<I> FinishInto<[u8; 32]> for Blake2x256Hasher<I>
where
    I: InputBuffer,
{
    fn finish_into(&self, output: &mut [u8; 32]) {
        super::blake2_256_raw_into(self.input.as_slice(), output)
    }
}

impl<I> FinishInto<[u8; 16]> for Blake2x128Hasher<I>
where
    I: InputBuffer,
{
    fn finish_into(&self, output: &mut [u8; 16]) {
        super::blake2_128_raw_into(self.input.as_slice(), output)
    }
}

impl<I> FinishInto<[u8; 32]> for TwoxHasher<I>
where
    I: InputBuffer,
{
    fn finish_into(&self, output: &mut [u8; 32]) {
        super::twox_256_raw_into(self.input.as_slice(), output)
    }
}

impl<I> FinishInto<[u8; 16]> for TwoxHasher<I>
where
    I: InputBuffer,
{
    fn finish_into(&self, output: &mut [u8; 16]) {
        super::twox_128_raw_into(self.input.as_slice(), output)
    }
}

impl<I> FinishInto<[u8; 8]> for TwoxHasher<I>
where
    I: InputBuffer,
{
    fn finish_into(&self, output: &mut [u8; 8]) {
        super::twox_64_raw_into(self.input.as_slice(), output)
    }
}

pub trait FinishU64 {
    fn finish(&self) -> u64;
}

impl<I> FinishU64 for Sha2x256Hasher<I>
where
    I: InputBuffer,
{
    fn finish(&self) -> u64 {
        let [h0, h1, h2, h3, h4, h5, h6, h7, ..] =
            <Self as Finish<[u8; 32]>>::finish(self);
        u64::from_le_bytes([h0, h1, h2, h3, h4, h5, h6, h7])
    }
}

impl<I> FinishU64 for Keccakx256Hasher<I>
where
    I: InputBuffer,
{
    fn finish(&self) -> u64 {
        let [h0, h1, h2, h3, h4, h5, h6, h7, ..] =
            <Self as Finish<[u8; 32]>>::finish(self);
        u64::from_le_bytes([h0, h1, h2, h3, h4, h5, h6, h7])
    }
}

impl<I> FinishU64 for Blake2x256Hasher<I>
where
    I: InputBuffer,
{
    fn finish(&self) -> u64 {
        let [h0, h1, h2, h3, h4, h5, h6, h7, ..] =
            <Self as Finish<[u8; 32]>>::finish(self);
        u64::from_le_bytes([h0, h1, h2, h3, h4, h5, h6, h7])
    }
}

impl<I> FinishU64 for Blake2x128Hasher<I>
where
    I: InputBuffer,
{
    fn finish(&self) -> u64 {
        let [h0, h1, h2, h3, h4, h5, h6, h7, ..] =
            <Self as Finish<[u8; 16]>>::finish(self);
        u64::from_le_bytes([h0, h1, h2, h3, h4, h5, h6, h7])
    }
}

impl<I> FinishU64 for TwoxHasher<I>
where
    I: InputBuffer,
{
    fn finish(&self) -> u64 {
        u64::from_le_bytes(<Self as Finish<[u8; 8]>>::finish(self))
    }
}

impl<T, I> Hasher for CryptoHasher<T, I>
where
    Self: FinishU64,
    T: markers::Sealed,
    I: InputBuffer,
{
    #[inline]
    fn finish(&self) -> u64 {
        <Self as FinishU64>::finish(self)
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        <I as InputBuffer>::write(&mut self.input, bytes)
    }
}

/// Default build hasher for supported cryptographic hashers.
pub type CryptoBuildHasher<T, I> = BuildHasherDefault<CryptoHasher<T, I>>;
