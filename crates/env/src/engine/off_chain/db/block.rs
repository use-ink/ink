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

use super::{
    super::{
        Result,
        TypedEncoded,
    },
    OffBlockNumber,
    OffHash,
    OffTimestamp,
};
use crate::Environment;

/// An emulated block in the chain.
pub struct Block {
    /// The current block number.
    number: OffBlockNumber,
    /// The timestamp of the block.
    timestamp: OffTimestamp,
    /// The randomization entropy for a block.
    ///
    /// # Note
    ///
    /// - Can optionally be set for more control via
    ///   [`crate::set_block_randomization_hash`].
    entropy: OffHash,
}

impl Block {
    /// Creates a new block for the given number and time stamp.
    pub fn new<T>(number: T::BlockNumber, timestamp: T::Timestamp) -> Self
    where
        T: Environment,
    {
        use crate::Clear;
        use rand::Rng as _;
        let mut entropy = <T as Environment>::Hash::clear();
        rand::thread_rng().fill(entropy.as_mut());
        Self {
            number: TypedEncoded::new(&number),
            timestamp: TypedEncoded::new(&timestamp),
            entropy: TypedEncoded::new(&entropy),
        }
    }

    /// Returns the block number.
    pub fn number<T>(&self) -> Result<T::BlockNumber>
    where
        T: Environment,
    {
        self.number.decode().map_err(Into::into)
    }

    /// Returns the timestamp of the block.
    pub fn timestamp<T>(&self) -> Result<T::Timestamp>
    where
        T: Environment,
    {
        self.timestamp.decode().map_err(Into::into)
    }

    /// Sets the entropy of this block to the given entropy.
    ///
    /// # Note
    ///
    /// This is mainly used to control what [`crate::random`] returns
    /// in the off-chain environment.
    pub fn set_entropy<T>(&mut self, new_entropy: T::Hash) -> Result<()>
    where
        T: Environment,
    {
        self.entropy.assign(&new_entropy).map_err(Into::into)
    }

    /// Returns a randomized hash.
    ///
    /// # Note
    ///
    /// - This is the off-chain environment implementation of
    /// [`crate::random`]. It provides the same behaviour in that it
    /// will likely yield the same hash for the same subjects within the same
    /// block (or execution context).
    ///
    /// - Returned hashes on the surface might appear random, however for
    /// testability purposes the actual implementation is quite simple and
    /// computes those "random" hashes by wrapping XOR of the internal entry hash
    /// with the eventually repeated sequence of the subject buffer.
    pub fn random<T>(&self, subject: &[u8]) -> Result<T::Hash>
    where
        T: Environment,
    {
        let mut entropy = self.entropy.clone();
        let entropy_bytes = entropy.encoded_bytes_mut()?;
        let len_entropy = entropy_bytes.len();
        for (n, subject) in subject.iter().enumerate() {
            let id = n % len_entropy;
            entropy_bytes[id] = entropy_bytes[id] ^ subject ^ (n as u8);
        }
        Ok(entropy.decode::<T::Hash>()?)
    }
}
