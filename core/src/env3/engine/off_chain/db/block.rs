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
};
use crate::env3::EnvTypes;

/// An emulated block in the chain.
pub struct Block {
    /// The current block number.
    number: OffBlockNumber,
    /// The current moment of block creation.
    moment: OffMoment,
}

impl Block {
    /// Creates a new block for the given number and moment.
    pub fn new<T>(number: T::BlockNumber, moment: T::Moment) -> Self
    where
        T: EnvTypes,
    {
        Self {
            number: TypedEncoded::new(&number),
            moment: TypedEncoded::new(&moment),
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
}
