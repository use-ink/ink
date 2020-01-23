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
    ext,
    EnvInstance,
};
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

impl EnvInstance {
    /// Empties the contract-side scratch buffer.
    ///
    /// # Note
    ///
    /// This is useful to perform before invoking a series of
    /// [`WasmEnv::append_encode_into_buffer`].
    fn reset_buffer(&mut self) {
        self.buffer.clear()
    }

    /// Reads the current scratch buffer into the contract-side buffer.
    ///
    /// Returns the amount of bytes read.
    fn read_scratch_buffer(&mut self) -> usize {
        let req_len = ext::scratch_size();
        self.buffer.resize(req_len, Default::default());
        ext::scratch_read(&mut self.buffer[0..req_len], 0);
        req_len
    }

    /// Reads from the scratch buffer and directly decodes into a value of `T`.
    ///
    /// # Errors
    ///
    /// If the decoding into a value of `T` failed.
    fn decode_scratch_buffer<T>(&mut self) -> core::result::Result<T, scale::Error>
    where
        T: scale::Decode,
    {
        let req_len = self.read_scratch_buffer();
        scale::Decode::decode(&mut &self.buffer[0..req_len])
    }

    /// Encodes the value into the contract-side scratch buffer.
    fn encode_into_buffer<T>(&mut self, value: T)
    where
        T: scale::Encode,
    {
        scale::Encode::encode_to(&value, &mut self.buffer);
    }

    /// Appends the encoded value into the contract-side scratch buffer
    /// and returns the byte ranges into the encoded region.
    fn append_encode_into_buffer<T>(&mut self, value: T) -> core::ops::Range<usize>
    where
        T: scale::Encode,
    {
        let start = self.buffer.len();
        scale::Encode::encode_to(&value, &mut self.buffer);
        let end = self.buffer.len();
        core::ops::Range { start, end }
    }

    /// Returns the contract property value.
    fn get_property<P>(&mut self, ext_fn: fn()) -> Result<P::In>
    where
        P: ReadProperty,
    {
        ext_fn();
        self.decode_scratch_buffer().map_err(Into::into)
    }

    /// Reusable implementation for invoking another contract message.
    fn invoke_contract_impl<T, RetType>(
        &mut self,
        call_params: &CallParams<T, RetType>,
    ) -> Result<()>
    where
        T: EnvTypes,
    {
        // Reset the contract-side buffer to append onto clean slate.
        self.reset_buffer();
        // Append the encoded `call_data`, `endowment` and `call_data`
        // in order and remember their encoded regions within the buffer.
        let callee = self.append_encode_into_buffer(call_params.callee());
        let endowment = self.append_encode_into_buffer(call_params.endowment());
        let call_data = self.append_encode_into_buffer(call_params.input_data());
        // Resolve the encoded regions into actual byte slices.
        let callee = &self.buffer[callee];
        let endowment = &self.buffer[endowment];
        let call_data = &self.buffer[call_data];
        // Perform the actual contract call.
        let ret = ext::call(callee, call_params.gas_limit(), endowment, call_data);
        if !ret.is_success() {
            // Return an error if `ret` refers to an error code.
            todo!()
        }
        Ok(())
    }
}

impl Env for EnvInstance {
    fn set_contract_storage<V>(&mut self, key: Key, value: &V)
    where
        V: scale::Encode,
    {
        self.encode_into_buffer(value);
        ext::set_storage(key.as_bytes(), &self.buffer);
    }

    fn get_contract_storage<R>(&mut self, key: Key) -> Result<R>
    where
        R: scale::Decode,
    {
        if !ext::get_storage(key.as_bytes()).is_success() {
            todo!()
        }
        self.decode_scratch_buffer().map_err(Into::into)
    }

    fn clear_contract_storage(&mut self, key: Key) {
        ext::clear_storage(key.as_bytes())
    }

    fn get_runtime_storage<R>(&mut self, runtime_key: &[u8]) -> Result<R>
    where
        R: scale::Decode,
    {
        if !ext::get_runtime_storage(runtime_key).is_success() {
            todo!()
        }
        self.decode_scratch_buffer().map_err(Into::into)
    }

    fn input(&mut self) -> Result<CallData> {
        self.get_property::<property::Input>(|| ())
    }

    fn output<R>(&mut self, return_value: &R)
    where
        R: scale::Encode,
    {
        self.encode_into_buffer(return_value);
        ext::scratch_write(&self.buffer);
    }

    fn println(&mut self, content: &str) {
        ext::println(content)
    }
}

impl TypedEnv for EnvInstance {
    fn caller<T: EnvTypes>(&mut self) -> Result<T::AccountId> {
        self.get_property::<property::Caller<T>>(ext::caller)
    }

    fn transferred_balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<property::TransferredBalance<T>>(ext::value_transferred)
    }

    fn gas_price<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<property::GasPrice<T>>(ext::gas_price)
    }

    fn gas_left<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<property::GasLeft<T>>(ext::gas_left)
    }

    fn now_in_ms<T: EnvTypes>(&mut self) -> Result<T::Moment> {
        self.get_property::<property::NowInMs<T>>(ext::now)
    }

    fn address<T: EnvTypes>(&mut self) -> Result<T::AccountId> {
        self.get_property::<property::Address<T>>(ext::address)
    }

    fn balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<property::Balance<T>>(ext::balance)
    }

    fn rent_allowance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<property::RentAllowance<T>>(ext::rent_allowance)
    }

    fn block_number<T: EnvTypes>(&mut self) -> Result<T::BlockNumber> {
        self.get_property::<property::BlockNumber<T>>(ext::block_number)
    }

    fn minimum_balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<property::MinimumBalance<T>>(ext::minimum_balance)
    }

    fn emit_event<T, Event>(&mut self, event: Event)
    where
        T: EnvTypes,
        Event: Topics<T> + scale::Encode,
    {
        // Reset the contract-side buffer to append onto clean slate.
        self.reset_buffer();
        // Append the encoded `topics` and the raw encoded `data`
        // in order and remember their encoded regions within the buffer.
        let topics = self.append_encode_into_buffer(event.topics());
        let data = self.append_encode_into_buffer(event);
        // Resolve the encoded regions into actual byte slices.
        let topics = &self.buffer[topics];
        let data = &self.buffer[data];
        // Do the actual depositing of the event.
        ext::deposit_event(topics, data);
    }

    fn set_rent_allowance<T>(&mut self, new_value: T::Balance)
    where
        T: EnvTypes,
    {
        self.encode_into_buffer(&new_value);
        ext::set_rent_allowance(&self.buffer)
    }

    fn invoke_runtime<T>(&mut self, call: &T::Call) -> Result<()>
    where
        T: EnvTypes
    {
        self.encode_into_buffer(call);
        ext::dispatch_call(&self.buffer);
        Ok(())
    }

    fn invoke_contract<T>(&mut self, call_params: &CallParams<T, ()>) -> Result<()>
    where
        T: EnvTypes,
    {
        self.invoke_contract_impl(call_params)
    }

    fn eval_contract<T, R>(
        &mut self,
        call_params: &CallParams<T, ReturnType<R>>,
    ) -> Result<R>
    where
        T: EnvTypes,
        R: scale::Decode,
    {
        self.invoke_contract_impl(call_params)?;
        self.decode_scratch_buffer().map_err(Into::into)
    }

    fn create_contract<T, C>(
        &mut self,
        params: &CreateParams<T, C>,
    ) -> Result<T::AccountId>
    where
        T: EnvTypes,
    {
        // Reset the contract-side buffer to append onto clean slate.
        self.reset_buffer();
        // Append the encoded `code_hash`, `endowment` and `create_data`
        // in order and remember their encoded regions within the buffer.
        let code_hash = self.append_encode_into_buffer(params.code_hash());
        let endowment = self.append_encode_into_buffer(params.endowment());
        let create_data = self.append_encode_into_buffer(params.input_data());
        // Resolve the encoded regions into actual byte slices.
        let code_hash = &self.buffer[code_hash];
        let endowment = &self.buffer[endowment];
        let create_data = &self.buffer[create_data];
        // Do the actual contract instantiation.
        let ret = ext::create(code_hash, params.gas_limit(), endowment, create_data);
        if !ret.is_success() {
            // Return an error if `ret` refers to an error code.
            todo!()
        }
        // At this point our contract instantiation was successful
        // and we can now fetch the returned data and decode it for
        // the result value.
        self.decode_scratch_buffer().map_err(Into::into)
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
        // Reset the contract-side buffer to append onto clean slate.
        self.reset_buffer();
        // Append the encoded `account_id`, `code_hash` and `rent_allowance`
        // and `filtered_keys` in order and remember their encoded regions
        // within the buffer.
        let account_id = self.append_encode_into_buffer(account_id);
        let code_hash = self.append_encode_into_buffer(code_hash);
        let rent_allowance = self.append_encode_into_buffer(rent_allowance);
        // Resolve the encoded regions into actual byte slices.
        let account_id = &self.buffer[account_id];
        let code_hash = &self.buffer[code_hash];
        let rent_allowance = &self.buffer[rent_allowance];
        // Perform the actual contract restoration.
        ext::restore_to(account_id, code_hash, rent_allowance, filtered_keys);
    }

    fn random<T>(&mut self, subject: &[u8]) -> Result<T::Hash>
    where
        T: EnvTypes,
    {
        ext::random_seed(subject);
        self.decode_scratch_buffer().map_err(Into::into)
    }
}
