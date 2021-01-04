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
        CallData,
        Result,
        TypedEncoded,
    },
    OffAccountId,
    OffBalance,
};
use crate::Environment;
use ink_prelude::vec::Vec;

pub type Bytes = Vec<u8>;

/// The context of a contract execution.
pub struct ExecContext {
    /// The caller of the contract execution.
    ///
    /// Might be user or another contract.
    pub caller: OffAccountId,
    /// The callee of the contract execution.
    pub callee: OffAccountId,
    /// The transferred value from caller to callee.
    pub transferred_value: OffBalance,
    /// The gas provided for the whole execution.
    pub gas: OffBalance,
    /// The inputs provided for the whole execution.
    ///
    /// # Note
    ///
    /// This includes selector and encoded arguments.
    pub call_data: CallData,
    /// The output of the contract execution.
    pub output: Option<Bytes>,
}

impl ExecContext {
    /// Constructs a new execution context.
    pub fn build<T>() -> ExecContextBuilder<T>
    where
        T: Environment,
    {
        ExecContextBuilder::new()
    }

    /// Returns the caller.
    pub fn caller<T>(&self) -> Result<T::AccountId>
    where
        T: Environment,
    {
        self.caller.decode().map_err(Into::into)
    }

    /// Returns the callee.
    pub fn callee<T>(&self) -> Result<T::AccountId>
    where
        T: Environment,
    {
        self.callee.decode().map_err(Into::into)
    }

    /// Returns the transferred value.
    pub fn transferred_value<T>(&self) -> Result<T::Balance>
    where
        T: Environment,
    {
        self.transferred_value.decode().map_err(Into::into)
    }

    /// Returns the gas.
    pub fn gas<T>(&self) -> Result<T::Balance>
    where
        T: Environment,
    {
        self.gas.decode().map_err(Into::into)
    }

    /// Returns the call data.
    #[allow(
        dead_code,
        // Needed as soon as we support to execute contracts
        // directly through the off-chain environment.
    )]
    pub fn call_data(&self) -> &CallData {
        &self.call_data
    }

    /// Returns the contract execution output.
    #[allow(
        dead_code,
        // Needed as soon as we support to execute contracts
        // directly through the off-chain environment.
    )]
    pub fn output(&self) -> Option<&Bytes> {
        self.output.as_ref()
    }
}

/// Builder for execution contexts.
pub struct ExecContextBuilder<T>
where
    T: Environment,
{
    /// The caller of the newly created execution context.
    caller: Option<T::AccountId>,
    /// The callee of the newly created execution context.
    callee: Option<T::AccountId>,
    /// The transferred value from caller to callee.
    transferred_value: Option<T::Balance>,
    /// The gas provided for the contract execution from caller to callee.
    gas: Option<T::Balance>,
    /// The inputs given to the contract execution.
    call_data: Option<CallData>,
}

impl<T> ExecContextBuilder<T>
where
    T: Environment,
{
    /// Constructs a new execution context builder.
    pub fn new() -> Self {
        Self {
            caller: None,
            callee: None,
            transferred_value: None,
            gas: None,
            call_data: None,
        }
    }

    /// Sets caller of the execution context.
    ///
    /// # Panics
    ///
    /// If there has already been set a caller.
    pub fn caller(mut self, caller: T::AccountId) -> Self {
        if self.caller.is_some() {
            panic!("already has a caller");
        }
        self.caller = Some(caller);
        self
    }

    /// Sets callee of the execution context.
    ///
    /// # Panics
    ///
    /// If there has already been set a callee.
    pub fn callee(mut self, callee: T::AccountId) -> Self {
        if self.callee.is_some() {
            panic!("already has a callee");
        }
        self.callee = Some(callee);
        self
    }

    /// Sets the provided gas for the execution.
    ///
    /// # Panics
    ///
    /// If there has already been set provided gas.
    pub fn gas(mut self, gas: T::Balance) -> Self {
        if self.gas.is_some() {
            panic!("already has provided gas");
        }
        self.gas = Some(gas);
        self
    }

    /// Sets the transferred value (endowment) for the execution.
    ///
    /// # Panics
    ///
    /// If there has already been set transferred value (endowment).
    pub fn transferred_value(mut self, transferred_value: T::Balance) -> Self {
        if self.transferred_value.is_some() {
            panic!("already has set transferred value (endowment)");
        }
        self.transferred_value = Some(transferred_value);
        self
    }

    /// Sets the call data for the execution.
    ///
    /// # Panics
    ///
    /// If there has already been set call data.
    pub fn call_data(mut self, call_data: CallData) -> Self {
        if self.call_data.is_some() {
            panic!("already has set call data");
        }
        self.call_data = Some(call_data);
        self
    }

    /// Finishes construction of execution context.
    ///
    /// # Panics
    ///
    /// If any parameter has not yet been set.
    pub fn finish(self) -> ExecContext {
        let caller = self.caller.expect("need a valid caller at this point");
        let callee = self.callee.expect("need a valid callee at this point");
        let transferred_value = self
            .transferred_value
            .expect("need a valid transferred value (endowment) at this point");
        let gas = self.gas.expect("need valid provided gas at this point");
        ExecContext {
            caller: TypedEncoded::new(&caller),
            callee: TypedEncoded::new(&callee),
            transferred_value: TypedEncoded::new(&transferred_value),
            gas: TypedEncoded::new(&gas),
            call_data: self.call_data.unwrap(),
            output: None,
        }
    }
}
