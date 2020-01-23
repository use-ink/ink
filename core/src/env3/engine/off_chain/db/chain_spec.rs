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

use super::OffBalance;
use crate::env3::EnvTypes;
use super::super::Result;

/// The chain specification.
pub struct ChainSpec {
    /// The current gas price.
    gas_price: OffBalance,
    /// The existential balance needed to create a tombstone upon contract eviction.
    existential_balance: OffBalance,
}

impl ChainSpec {
    /// Creates a new uninitialized chain specification.
    pub fn uninitialized() -> Self {
        Self {
            gas_price: OffBalance::uninitialized(),
            existential_balance: OffBalance::uninitialized(),
        }
    }

    /// Returns the gas price for the chain.
    pub fn gas_price<T>(&self) -> Result<T::Balance>
    where
        T: EnvTypes,
    {
        self.gas_price.decode().map_err(Into::into)
    }

    /// Returns the existential balance for the chain.
    pub fn existential_balance<T>(&self) -> Result<T::Balance>
    where
        T: EnvTypes,
    {
        self.existential_balance.decode().map_err(Into::into)
    }
}
