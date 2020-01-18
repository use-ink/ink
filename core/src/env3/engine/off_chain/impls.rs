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

use super::TestEnv;
use crate::{
    env3::{
        call::{
            CallData,
            CallParams,
            CreateParams,
            ReturnType,
        },
        property,
        property::ReadProperty,
        Env,
        EnvTypes,
        Result,
        Topics,
        TypedEnv,
    },
    storage::Key,
};

impl Env for TestEnv {
    fn set_contract_storage<V>(&mut self, key: Key, value: &V)
    where
        V: scale::Encode,
    {
        todo!()
    }

    fn get_contract_storage<R>(&mut self, key: Key) -> Result<R>
    where
        R: scale::Decode,
    {
        todo!()
    }

    fn clear_contract_storage(&mut self, key: Key) {
        todo!()
    }

    fn get_runtime_storage<R>(&mut self, runtime_key: &[u8]) -> Result<R>
    where
        R: scale::Decode,
    {
        todo!()
    }

    fn input(&mut self) -> Result<CallData> {
        todo!()
    }

    fn output<R>(&mut self, return_value: &R)
    where
        R: scale::Encode,
    {
        todo!()
    }

    fn println(&mut self, content: &str) {
        println!("{}", content)
    }
}

impl TypedEnv for WasmEnv {
    fn caller<T: EnvTypes>(&mut self) -> Result<T::AccountId> {
        todo!()
    }

    fn transferred_balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        todo!()
    }

    fn gas_price<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        todo!()
    }

    fn gas_left<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        todo!()
    }

    fn now_in_ms<T: EnvTypes>(&mut self) -> Result<T::Moment> {
        todo!()
    }

    fn address<T: EnvTypes>(&mut self) -> Result<T::AccountId> {
        todo!()
    }

    fn balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        todo!()
    }

    fn rent_allowance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        todo!()
    }

    fn block_number<T: EnvTypes>(&mut self) -> Result<T::BlockNumber> {
        todo!()
    }

    fn minimum_balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        todo!()
    }

    fn emit_event<T, Event>(&mut self, event: Event)
    where
        T: EnvTypes,
        Event: Topics<T> + scale::Encode,
    {
        todo!()
    }

    fn set_rent_allowance<T>(&mut self, new_value: T::Balance)
    where
        T: EnvTypes,
    {
        todo!()
    }

    fn invoke_contract<T>(&mut self, call_params: &CallParams<T, ()>) -> Result<()>
    where
        T: EnvTypes,
    {
        todo!()
    }

    fn eval_contract<T, R>(
        &mut self,
        call_params: &CallParams<T, ReturnType<R>>,
    ) -> Result<R>
    where
        T: EnvTypes,
        R: scale::Decode,
    {
        todo!()
    }

    fn create_contract<T, C>(
        &mut self,
        params: &CreateParams<T, C>,
    ) -> Result<T::AccountId>
    where
        T: EnvTypes,
    {
        todo!()
    }

    fn restore_contract<T>(
        &mut self,
        account_id: T::AccountId,
        code_hash: T::Hash,
        rent_allowance: T::Balance,
        filtered_keys: &[Key],
    ) where
        T: EnvTypes,
    {
        todo!()
    }

    fn random<T>(&mut self, subject: &[u8]) -> Result<T::Hash>
    where
        T: EnvTypes,
    {
        todo!()
    }
}
