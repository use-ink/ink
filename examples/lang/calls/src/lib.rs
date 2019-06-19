// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(any(test, feature = "test-env")), no_std)]

use ink_core::{
    env::DefaultSrmlTypes,
};
use ink_lang::contract;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    /// This simple dummy contract dispatches substrate runtime calls
    struct Calls {}

    impl Deploy for Calls {
        fn deploy(&mut self) {
        }
    }

    impl Calls {
        /// Dispatches a `transfer` call to the Balances srml module
        pub(external) fn balance_transfer(&mut self, dest: u32, value: Balance) {
            let transfer_call = ink_core::env::calls::Balances::<DefaultSrmlTypes>::transfer(dest, value);
            unsafe { ink_core::env::dispatch_call::<ink_core::env::ContractEnv<DefaultSrmlTypes>>(transfer_call.into()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatches_balances_call() {
        let mut calls = Calls::deploy_mock();
        assert_eq!(env::dispatched_calls().into_iter().count(), 0);
        calls.balance_transfer(1, 10000);
        assert_eq!(env::dispatched_calls().into_iter().count(), 1);
    }
}
