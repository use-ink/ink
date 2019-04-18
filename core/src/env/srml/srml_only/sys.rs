// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

//! External C API to communicate with substrate contracts runtime module.
//!
//! Refer to substrate SRML contract module for more documentation.

extern "C" {
    /// Creates a new smart contract account.
    #[allow(unused)]
    pub fn ext_create(
        init_code_ptr: u32,
        init_code_len: u32,
        gas: u64,
        value_ptr: u32,
        value_len: u32,
        input_data_ptr: u32,
        input_data_len: u32,
    ) -> u32;

    /// Calls a remote smart contract.
    #[allow(unused)]
    pub fn ext_call(
        callee_ptr: u32,
        callee_len: u32,
        gas: u64,
        value_ptr: u32,
        value_len: u32,
        input_data_ptr: u32,
        input_data_len: u32,
    ) -> u32;

    /// Tells the execution environment to load the raw byte
    /// representation of the caller into the scratch buffer.
    pub fn ext_caller();

    /// Prints the contents of `str_ptr` to the console.
    ///
    /// # Note
    ///
    /// This must only be used in `--dev` chain environments!
    pub fn ext_println(str_ptr: u32, str_len: u32);

    /// Deposits raw event data through the Contracts module.
    pub fn ext_deposit_event(data_ptr: u32, data_len: u32);

    /// Writes the contents of the buffer at `value_ptr` into the
    /// storage slot associated with the given key or clears the
    /// associated slot if `value_non_null` is `0`.
    pub fn ext_set_storage(
        key_ptr: u32,
        value_non_null: u32,
        value_ptr: u32,
        value_len: u32,
    );

    /// Tells the execution environment to load the contents
    /// stored at the given key into the scratch buffer.
    pub fn ext_get_storage(key_ptr: u32) -> u32;

    /// Returns the length in bytes of the scratch buffer.
    pub fn ext_scratch_size() -> u32;

    /// Copies the contents of the scratch buffer to `dest_ptr`.
    pub fn ext_scratch_copy(dest_ptr: u32, offset: u32, len: u32);

    /// Returns the length of the input buffer.
    pub fn ext_input_size() -> u32;

    /// Copies the contents of the input buffer to `dest_ptr`.
    pub fn ext_input_copy(dest_ptr: u32, offset: u32, len: u32);

    /// Immediately returns contract execution to the caller
    /// with the provided data at `data_ptr`.
    pub fn ext_return(data_ptr: u32, data_len: u32) -> !;

    /// Stores the address of the current contract into the scratch buffer.
    pub fn ext_address();

    // Stores the gas price for the current transaction into the scratch buffer.
    //
    // The data is encoded as T::Balance. The current contents of the scratch buffer are overwritten.
    pub fn ext_gas_price();

    // Stores the amount of gas left into the scratch buffer.
    //
    // The data is encoded as T::Balance. The current contents of the scratch buffer are overwritten.
    pub fn ext_gas_left();

    // Stores the balance of the current account into the scratch buffer.
    //
    // The data is encoded as T::Balance. The current contents of the scratch buffer are overwritten.
    pub fn ext_balance();

    // Stores the value transferred along with this call or as endowment into the scratch buffer.
    //
    // The data is encoded as T::Balance. The current contents of the scratch buffer are overwritten.
    pub fn ext_value_transferred();

    // Load the latest block RNG seed into the scratch buffer.
    pub fn ext_random_seed();

    // Load the latest block timestamp into the scratch buffer.
    pub fn ext_now();
}
