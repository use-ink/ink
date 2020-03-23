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
    /// Prevents users from implementing certain traits or generic types.
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
pub struct CryptoHasher<T, A>
where
    T: markers::Sealed,
{
    /// The accumulator buffer.
    ///
    /// A bytes buffer that is used to accumulate a hashing state.
    accumulator: A,
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

/// Types that qualify as accumulator.
///
/// # Examples
///
/// Rust's `Vec<u8>` types and an exclusive reference to it (`&mut Vec<u8>`)
/// qualify as such. Users may implement this trait for other types. E.g. it
/// could be useful to have a `SmallVec` or a static buffer implementation for
/// this trait.
pub trait Accumulator {
    /// Resets the buffer which cleans all state from it.
    ///
    /// # Note
    ///
    /// Useful when using `Vec` or similar as accumulator.
    fn reset(&mut self);
    /// Writes the given bytes into the buffer.
    fn write(&mut self, bytes: &[u8]);
    /// Returns a shared reference to the slice of the current state of the buffer.
    fn as_slice(&self) -> &[u8];
}

impl Accumulator for Vec<u8> {
    fn reset(&mut self) {
        <Vec<_>>::clear(self)
    }

    fn write(&mut self, bytes: &[u8]) {
        // This could theoretically be speed-up by using `unsafe` `set_len`
        // and `[u8]` `copy_from_slice` methods.
        <Vec<_>>::extend_from_slice(self, bytes)
    }

    fn as_slice(&self) -> &[u8] {
        <Vec<_>>::as_slice(self)
    }
}

impl<'a, T> Accumulator for &'a mut T
where
    T: Accumulator,
{
    fn reset(&mut self) {
        <T as Accumulator>::reset(self)
    }

    fn write(&mut self, bytes: &[u8]) {
        <T as Accumulator>::write(self, bytes)
    }

    fn as_slice(&self) -> &[u8] {
        <T as Accumulator>::as_slice(self)
    }
}

/// Wraps a bytes buffer and turns it into an accumulator.
///
/// # Panics
///
/// Upon hash calculation if the underlying buffer length does not suffice the
/// needs of the accumulated hash buffer.
pub struct Wrap<'a> {
    /// The underlying wrapped buffer.
    buffer: &'a mut [u8],
    /// The current length of the filled area.
    len: usize,
}

impl<'a> From<&'a mut [u8]> for Wrap<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { buffer, len: 0 }
    }
}

impl<'a> Accumulator for Wrap<'a> {
    fn reset(&mut self) {
        self.len = 0;
    }

    fn write(&mut self, bytes: &[u8]) {
        let len = self.len;
        let bytes_len = bytes.len();
        <[u8]>::copy_from_slice(&mut self.buffer[len..(len + bytes_len)], bytes);
        self.len += bytes_len;
    }

    fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.len]
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

impl<T, A> From<A> for CryptoHasher<T, A>
where
    T: markers::Sealed,
    A: Accumulator,
{
    fn from(mut accumulator: A) -> Self {
        <A as Accumulator>::reset(&mut accumulator);
        Self {
            accumulator,
            marker: PhantomData,
        }
    }
}

/// Hash functions that can put their results into an output buffer
/// described by the `Output` type.
///
/// # Note
///
/// This is an internal trait used to deduplicate some implementations.
pub trait FinishInto<Output> {
    fn finish_into(&self, output: &mut Output);
}

/// Hash functions that can return their hash results as an output buffer
/// described by the `Output` type.
///
/// # Note
///
/// - Do not manually implement this trait since it is automatically implemented
///   as `Finish<O>` for all `T: FinishInto<O>`.
/// - This is an internal trait used to deduplicate some implementations.
pub trait Finish<Output> {
    fn finish(&self) -> Output;
}

impl<T, A, O> Finish<O> for CryptoHasher<T, A>
where
    Self: FinishInto<O>,
    T: markers::Sealed,
    A: Accumulator,
    O: AsMut<[u8]> + Default,
{
    fn finish(&self) -> O {
        let mut output = <O as Default>::default();
        <Self as FinishInto<O>>::finish_into(self, &mut output);
        output
    }
}

impl<A> FinishInto<[u8; 32]> for Sha2x256Hasher<A>
where
    A: Accumulator,
{
    fn finish_into(&self, output: &mut [u8; 32]) {
        super::sha2_256_raw_into(self.accumulator.as_slice(), output)
    }
}

impl<A> FinishInto<[u8; 32]> for Keccakx256Hasher<A>
where
    A: Accumulator,
{
    fn finish_into(&self, output: &mut [u8; 32]) {
        super::keccak_256_raw_into(self.accumulator.as_slice(), output)
    }
}

impl<A> FinishInto<[u8; 32]> for Blake2x256Hasher<A>
where
    A: Accumulator,
{
    fn finish_into(&self, output: &mut [u8; 32]) {
        super::blake2_256_raw_into(self.accumulator.as_slice(), output)
    }
}

impl<A> FinishInto<[u8; 16]> for Blake2x128Hasher<A>
where
    A: Accumulator,
{
    fn finish_into(&self, output: &mut [u8; 16]) {
        super::blake2_128_raw_into(self.accumulator.as_slice(), output)
    }
}

impl<A> FinishInto<[u8; 32]> for TwoxHasher<A>
where
    A: Accumulator,
{
    fn finish_into(&self, output: &mut [u8; 32]) {
        super::twox_256_raw_into(self.accumulator.as_slice(), output)
    }
}

impl<A> FinishInto<[u8; 16]> for TwoxHasher<A>
where
    A: Accumulator,
{
    fn finish_into(&self, output: &mut [u8; 16]) {
        super::twox_128_raw_into(self.accumulator.as_slice(), output)
    }
}

impl<A> FinishInto<[u8; 8]> for TwoxHasher<A>
where
    A: Accumulator,
{
    fn finish_into(&self, output: &mut [u8; 8]) {
        super::twox_64_raw_into(self.accumulator.as_slice(), output)
    }
}

/// Crypto hash functions that allow to return their hash as `u64`.
///
/// # Note
///
/// - This is an internal trait used to deduplicate some implementations.
/// - This is a compatibility function for direct usage of the supported
///   crypto hash functions via [`Hash`](`core::hash::Hash`) trait.
pub trait FinishU64 {
    /// Returns the `u64` result of the accumulated hash.
    fn finish(&self) -> u64;
}

fn truncate_u8x32_to_u64(bytes: [u8; 32]) -> u64 {
    let [h0, h1, h2, h3, h4, h5, h6, h7, ..] = bytes;
    u64::from_le_bytes([h0, h1, h2, h3, h4, h5, h6, h7])
}

fn truncate_u8x16_to_u64(bytes: [u8; 16]) -> u64 {
    let [h0, h1, h2, h3, h4, h5, h6, h7, ..] = bytes;
    u64::from_le_bytes([h0, h1, h2, h3, h4, h5, h6, h7])
}

impl<A> FinishU64 for Sha2x256Hasher<A>
where
    A: Accumulator,
{
    fn finish(&self) -> u64 {
        truncate_u8x32_to_u64(<Self as Finish<[u8; 32]>>::finish(self))
    }
}

impl<A> FinishU64 for Keccakx256Hasher<A>
where
    A: Accumulator,
{
    fn finish(&self) -> u64 {
        truncate_u8x32_to_u64(<Self as Finish<[u8; 32]>>::finish(self))
    }
}

impl<A> FinishU64 for Blake2x256Hasher<A>
where
    A: Accumulator,
{
    fn finish(&self) -> u64 {
        truncate_u8x32_to_u64(<Self as Finish<[u8; 32]>>::finish(self))
    }
}

impl<A> FinishU64 for Blake2x128Hasher<A>
where
    A: Accumulator,
{
    fn finish(&self) -> u64 {
        truncate_u8x16_to_u64(<Self as Finish<[u8; 16]>>::finish(self))
    }
}

impl<A> FinishU64 for TwoxHasher<A>
where
    A: Accumulator,
{
    fn finish(&self) -> u64 {
        u64::from_le_bytes(<Self as Finish<[u8; 8]>>::finish(self))
    }
}

impl<T, A> Hasher for CryptoHasher<T, A>
where
    Self: FinishU64,
    T: markers::Sealed,
    A: Accumulator,
{
    #[inline]
    fn finish(&self) -> u64 {
        <Self as FinishU64>::finish(self)
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        <A as Accumulator>::write(&mut self.accumulator, bytes)
    }
}

/// Default build hasher for supported cryptographic hashers.
pub type CryptoBuildHasher<T, A> = BuildHasherDefault<CryptoHasher<T, A>>;
