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

pub use ink_primitives_derive::Storable;

/// Trait for representing types which can be read and written to storage.
///
/// This trait is not the same as the `scale::Encode + scale::Decode`. Each type that implements
/// `scale::Encode + scale::Decode` are storable by default and transferable between contracts.
/// But not each storable type is transferable.
pub trait Storable: Sized {
    /// Convert self to a slice and append it to the destination.
    fn encode<T: scale::Output + ?Sized>(&self, dest: &mut T);

    /// Attempt to deserialize the value from input.
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error>;
}

/// Types which implement `scale::Encode` and `scale::Decode` are `Storable` by default because
/// they can be written directly into the storage cell.
impl<P> Storable for P
where
    P: scale::Encode + scale::Decode,
{
    #[inline]
    fn encode<T: scale::Output + ?Sized>(&self, dest: &mut T) {
        scale::Encode::encode_to(self, dest)
    }

    #[inline]
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        scale::Decode::decode(input)
    }
}
