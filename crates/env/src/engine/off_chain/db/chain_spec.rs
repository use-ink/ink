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
    super::Result,
    OffBalance,
    OffTimestamp,
};
use crate::Environment;

/// The chain specification.
pub struct ChainSpec {
    /// The current gas price.
    gas_price: OffBalance,
    /// The minimum value an account of the chain may have.
    minimum_balance: OffBalance,
    /// The tombstone deposit.
    tombstone_deposit: OffBalance,
    /// The targeted block time.
    block_time: OffTimestamp,
}

impl ChainSpec {
    /// Creates a new uninitialized chain specification.
    pub fn uninitialized() -> Self {
        Self {
            gas_price: OffBalance::uninitialized(),
            minimum_balance: OffBalance::uninitialized(),
            tombstone_deposit: OffBalance::uninitialized(),
            block_time: OffTimestamp::uninitialized(),
        }
    }

    /// Resets the chain spec to uninitialized state.
    pub fn reset(&mut self) {
        self.gas_price = OffBalance::uninitialized();
        self.minimum_balance = OffBalance::uninitialized();
        self.tombstone_deposit = OffBalance::uninitialized();
        self.block_time = OffTimestamp::uninitialized();
    }

    /// Default initialization for the off-chain specification.
    pub fn initialize_as_default<T>(&mut self) -> crate::Result<()>
    where
        T: Environment,
        <T as Environment>::AccountId: From<[u8; 32]>,
    {
        self.gas_price
            .try_initialize::<T::Balance>(&T::Balance::from(100u32))?;
        self.minimum_balance
            .try_initialize::<T::Balance>(&T::Balance::from(42u32))?;
        self.tombstone_deposit
            .try_initialize::<T::Balance>(&T::Balance::from(16u32))?;
        self.block_time
            .try_initialize::<T::Timestamp>(&T::Timestamp::from(5u32))?;
        Ok(())
    }

    /// Returns the gas price for the chain.
    pub fn gas_price<T>(&self) -> Result<T::Balance>
    where
        T: Environment,
    {
        self.gas_price.decode().map_err(Into::into)
    }

    /// Set the gas price for the chain.
    pub fn set_gas_price<T>(&mut self, gas_price: T::Balance)
    where
        T: Environment,
    {
        self.gas_price = OffBalance::new(&gas_price)
    }

    /// Returns the minimum balance that is required for creating an account.
    pub fn minimum_balance<T>(&self) -> Result<T::Balance>
    where
        T: Environment,
    {
        self.minimum_balance.decode().map_err(Into::into)
    }

    /// Returns the tombstone deposit for the chain.
    pub fn tombstone_deposit<T>(&self) -> Result<T::Balance>
    where
        T: Environment,
    {
        self.tombstone_deposit.decode().map_err(Into::into)
    }

    /// Returns the targeted block time for the chain.
    pub fn block_time<T>(&self) -> Result<T::Timestamp>
    where
        T: Environment,
    {
        self.block_time.decode().map_err(Into::into)
    }
}
