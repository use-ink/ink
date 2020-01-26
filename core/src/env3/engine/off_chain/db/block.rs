// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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
    super::{
        Result,
        TypedEncoded,
    },
    OffBlockNumber,
    OffMoment,
    OffHash,
};
use crate::env3::EnvTypes;

/// An emulated block in the chain.
pub struct Block {
    /// The current block number.
    number: OffBlockNumber,
    /// The current moment of block creation.
    moment: OffMoment,
    /// The randomization entropy for a block.
    ///
    /// # Note
    ///
    /// - Can optionally be set for more control via
    ///   [`crate::env3::set_block_randomization_hash`].
    entropy: OffHash,
}

impl Block {
    /// Creates a new block for the given number and moment.
    pub fn new<T>(number: T::BlockNumber, moment: T::Moment, entropy: T::Hash) -> Self
    where
        T: EnvTypes,
    {
        Self {
            number: TypedEncoded::new(&number),
            moment: TypedEncoded::new(&moment),
            entropy: TypedEncoded::new(&entropy),
        }
    }

    /// Returns the block number.
    pub fn number<T>(&self) -> Result<T::BlockNumber>
    where
        T: EnvTypes,
    {
        self.number.decode().map_err(Into::into)
    }

    /// Returns the moment of the block.
    pub fn moment<T>(&self) -> Result<T::Moment>
    where
        T: EnvTypes,
    {
        self.moment.decode().map_err(Into::into)
    }

    /// Sets the entropy of this block to the given entropy.
    ///
    /// # Note
    ///
    /// This is mainly used to control what [`crate::env3::random`] returns
    /// in the off-chain environment.
    pub fn set_entropy<T>(&mut self, new_entropy: T::Hash) -> Result<()>
    where
        T: EnvTypes,
    {
        self.entropy.assign(&new_entropy)
            .map_err(Into::into)
    }

    /// Returns a randomized hash.
    ///
    /// # Note
    ///
    /// - This is the off-chain environment implementation of
    /// [`crate::env3::random`]. It provides the same behaviour in that it
    /// will likely yield the same hash for the same subjects within the same
    /// block (or execution context).
    ///
    /// - Returned hashes on the surface might appear random, however for
    /// testability purposes the actual implementation is quite simple and
    /// computes those "random" hashes by wrapping XOR of the internal entry hash
    /// with the eventually repeated sequence of the subject buffer.
    pub fn random<T>(&self, subject: &[u8]) -> Result<T::Hash>
    where
        T: EnvTypes,
    {
        let mut entropy = self.entropy.clone();
        let mut entropy_bytes = entropy.encoded_bytes_mut()?;
        let mut len_entropy = entropy_bytes.len();
        for (n, subject) in subject.iter().enumerate() {
            let id = n % len_entropy;
            entropy_bytes[id] = entropy_bytes[id] ^ subject ^ (n as u8);
        }
        Ok(entropy.decode::<T::Hash>()?)
    }
}
