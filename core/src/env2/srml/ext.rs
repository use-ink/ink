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

//! External C API to communicate with substrate contracts runtime module.
//!
//! Refer to substrate SRML contract module for more documentation.

use crate::{
    env2::srml::RetCode,
    storage::Key,
};

pub fn create(
    code_hash: &[u8],
    gas_limit: u64,
    value: &[u8],
    create_data: &[u8],
) -> RetCode {
    unsafe {
        sys::ext_instantiate(
            code_hash.as_ptr() as u32,
            code_hash.len() as u32,
            gas_limit,
            value.as_ptr() as u32,
            value.len() as u32,
            create_data.as_ptr() as u32,
            create_data.len() as u32,
        )
    }
    .into()
}

pub fn call(callee: &[u8], gas_limit: u64, value: &[u8], call_data: &[u8]) -> RetCode {
    unsafe {
        sys::ext_call(
            callee.as_ptr() as u32,
            callee.len() as u32,
            gas_limit,
            value.as_ptr() as u32,
            value.len() as u32,
            call_data.as_ptr() as u32,
            call_data.len() as u32,
        )
    }
    .into()
}

pub fn deposit_event(topics: &[u8], data: &[u8]) {
    unsafe {
        sys::ext_deposit_event(
            topics.as_ptr() as u32,
            topics.len() as u32,
            data.as_ptr() as u32,
            data.len() as u32,
        )
    }
}

pub fn set_storage(key: &[u8], value: Option<&[u8]>) {
    match value {
        Some(value) => unsafe {
            sys::ext_set_storage(
                key.as_ptr() as u32,
                1,
                value.as_ptr() as u32,
                value.len() as u32,
            )
        },
        None => unsafe { sys::ext_set_storage(key.as_ptr() as u32, 0, 0, 0) },
    }
}

pub fn get_storage(key: &[u8]) -> RetCode {
    unsafe { sys::ext_get_storage(key.as_ptr() as u32) }.into()
}

pub fn restore_to(
    dest: &[u8],
    code_hash: &[u8],
    rent_allowance: &[u8],
    filtered_keys: &[Key],
) {
    unsafe {
        sys::ext_restore_to(
            dest.as_ptr() as u32,
            dest.len() as u32,
            code_hash.as_ptr() as u32,
            code_hash.len() as u32,
            rent_allowance.as_ptr() as u32,
            rent_allowance.len() as u32,
            filtered_keys.as_ptr() as u32,
            filtered_keys.len() as u32,
        )
    }
}

pub fn dispatch_call(call: &[u8]) {
    unsafe { sys::ext_dispatch_call(call.as_ptr() as u32, call.len() as u32) }
}

pub fn scratch_size() -> usize {
    (unsafe { sys::ext_scratch_size() }) as usize
}

pub fn scratch_read(dest: &mut [u8], offset: u32) -> RetCode {
    unsafe { sys::ext_scratch_read(dest.as_mut_ptr() as u32, offset, dest.len() as u32) };
    RetCode::success()
}

pub fn scratch_write(src: &[u8]) -> RetCode {
    unsafe { sys::ext_scratch_write(src.as_ptr() as u32, src.len() as u32) };
    RetCode::success()
}

macro_rules! impl_ext_wrapper_for {
    ( $( ($name:ident => $ext_name:ident), )* ) => {
        $(
            pub fn $name() {
                unsafe {
                    sys::$ext_name()
                }
            }
        )*
    }
}
impl_ext_wrapper_for! {
    (caller => ext_caller),
    (block_number => ext_block_number),
    (address => ext_address),
    (balance => ext_balance),
    (gas_price => ext_gas_price),
    (gas_left => ext_gas_left),
    (value_transferred => ext_value_transferred),
    (now => ext_now),
    (rent_allowance => ext_rent_allowance),
    (minimum_balance => ext_minimum_balance),
}

pub fn set_rent_allowance(value: &[u8]) -> RetCode {
    unsafe { sys::ext_set_rent_allowance(value.as_ptr() as u32, value.len() as u32) };
    RetCode::success()
}

pub fn random_seed(subject: &[u8]) {
    unsafe { sys::ext_random_seed(subject.as_ptr() as u32, subject.len() as u32) }
}

pub fn println(content: &str) {
    let bytes = content.as_bytes();
    unsafe { sys::ext_println(bytes.as_ptr() as u32, bytes.len() as u32) }
}

mod sys {
    extern "C" {
        pub fn ext_instantiate(
            init_code_ptr: u32,
            init_code_len: u32,
            gas: u64,
            value_ptr: u32,
            value_len: u32,
            input_data_ptr: u32,
            input_data_len: u32,
        ) -> u32;

        pub fn ext_call(
            callee_ptr: u32,
            callee_len: u32,
            gas: u64,
            value_ptr: u32,
            value_len: u32,
            input_data_ptr: u32,
            input_data_len: u32,
        ) -> u32;

        pub fn ext_deposit_event(
            topics_ptr: u32,
            topics_len: u32,
            data_ptr: u32,
            data_len: u32,
        );

        pub fn ext_set_storage(
            key_ptr: u32,
            value_non_null: u32,
            value_ptr: u32,
            value_len: u32,
        );
        pub fn ext_get_storage(key_ptr: u32) -> u32;

        pub fn ext_restore_to(
            dest_ptr: u32,
            dest_len: u32,
            code_hash_ptr: u32,
            code_hash_len: u32,
            rent_allowance_ptr: u32,
            rent_allowance_len: u32,
            delta_ptr: u32,
            delta_count: u32,
        );

        pub fn ext_dispatch_call(call_ptr: u32, call_len: u32);

        pub fn ext_scratch_size() -> u32;
        pub fn ext_scratch_read(dst_ptr: u32, offset: u32, len: u32);
        pub fn ext_scratch_write(src_ptr: u32, len: u32);

        pub fn ext_caller();
        pub fn ext_block_number();
        pub fn ext_address();
        pub fn ext_balance();
        pub fn ext_gas_price();
        pub fn ext_gas_left();
        pub fn ext_value_transferred();
        pub fn ext_now();
        pub fn ext_rent_allowance();
        pub fn ext_minimum_balance();

        pub fn ext_set_rent_allowance(value_ptr: u32, value_len: u32);

        pub fn ext_random_seed(subject_ptr: u32, subject_len: u32);
        pub fn ext_println(str_ptr: u32, str_len: u32);
    }
}
