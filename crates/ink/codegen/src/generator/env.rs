// Copyright (C) Use Ink (UK) Ltd.
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

use crate::GenerateCode;
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates code for the ink! environment of the contract.
#[derive(From)]
pub struct Env<'a> {
    contract: &'a ir::Contract,
}

impl GenerateCode for Env<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let env = self.contract.config().env();
        let storage_ident = self.contract.module().storage().ident();
        quote! {


pub mod write_to {
	use core::cmp::min;
	use core::fmt;

	pub struct WriteTo<'a> {
		buffer: &'a mut [u8],
		// on write error (i.e. not enough space in buffer) this grows beyond
		// `buffer.len()`.
		used: usize,
	}

	impl<'a> WriteTo<'a> {
		pub fn new(buffer: &'a mut [u8]) -> Self {
			WriteTo { buffer, used: 0 }
		}

		pub fn as_str(self) -> Option<&'a str> {
			if self.used <= self.buffer.len() {
				// only successful concats of str - must be a valid str.
				use core::str::from_utf8_unchecked;
				Some(unsafe { from_utf8_unchecked(&self.buffer[..self.used]) })
			} else {
				None
			}
		}
	}

	impl<'a> fmt::Write for WriteTo<'a> {
		fn write_str(&mut self, s: &str) -> fmt::Result {
			if self.used > self.buffer.len() {
				return Err(fmt::Error);
			}
			let remaining_buf = &mut self.buffer[self.used..];
			let raw_s = s.as_bytes();
			let write_num = min(raw_s.len(), remaining_buf.len());
			remaining_buf[..write_num].copy_from_slice(&raw_s[..write_num]);
			self.used += raw_s.len();
			if write_num < raw_s.len() {
				Err(fmt::Error)
			} else {
				Ok(())
			}
		}
	}

	pub fn show<'a>(buffer: &'a mut [u8], args: fmt::Arguments) -> Result<&'a str, fmt::Error> {
		let mut w = WriteTo::new(buffer);
		fmt::write(&mut w, args)?;
		w.as_str().ok_or(fmt::Error)
	}
}


#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum LocalBarPlain {
    /// API call successful.
    Success = 0,
    /// The called function trapped and has its state changes reverted.
    /// In this case no output buffer is returned.
    /// Can only be returned from `call` and `instantiate`.
    CalleeTrapped = 1,
    /// Returns if an unknown error was received from the host module.
    Unknown,
}

impl ::core::fmt::Debug for LocalBarPlain {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                LocalBarPlain::Success => "Success",
                LocalBarPlain::CalleeTrapped => "CalleeTrapped",
                LocalBarPlain::Unknown => "Unknown",
            },
        )
    }
}
            impl ::ink::env::ContractEnv for #storage_ident {
                type Env = #env;
            }

            type Environment = <#storage_ident as ::ink::env::ContractEnv>::Env;

            type AccountId = <<#storage_ident as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::AccountId;
            type Balance = <<#storage_ident as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Balance;
            type Hash = <<#storage_ident as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Hash;
            type Timestamp = <<#storage_ident as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Timestamp;
            type BlockNumber = <<#storage_ident as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::BlockNumber;
            type ChainExtension = <<#storage_ident as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::ChainExtension;
            const MAX_EVENT_TOPICS: usize = <<#storage_ident as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::MAX_EVENT_TOPICS;
            type EventRecord = <<#storage_ident as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::EventRecord;
        }
    }
}
