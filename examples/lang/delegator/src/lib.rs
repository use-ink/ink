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

#![cfg_attr(not(feature = "std"), no_std)]

use ink_core::{
    memory::format,
    storage,
};
use ink_lang::contract;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
pub enum Which {
    Adder,
    Subber,
}

impl ink_core::storage::Flush for Which {}

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    /// Delegates calls to an adder or subber contract to mutate
    /// a value in an accumulator contract.
    ///
    /// In order to deploy the delegator smart contract we first
    /// have to manually put the code of the accumulator, adder
    /// and subber smart contracts, receive their code hashes from
    /// the signalled events and put their code hash into our
    /// delegator smart contract.
    struct Delegator {
        /// Says which of adder or subber is currently in use.
        which: storage::Value<Which>,
        /// The accumulator smart contract.
        accumulator: storage::Value<accumulator::Accumulator>,
        /// The adder smart contract.
        adder: storage::Value<adder::Adder>,
        /// The subber smart contract.
        subber: storage::Value<subber::Subber>,
    }

    impl Deploy for Delegator {
        /// Initializes the value to the initial value.
        fn deploy(
            &mut self,
            init_value: i32,
            accumulator_code_hash: Hash,
            adder_code_hash: Hash,
            subber_code_hash: Hash,
        ) {
            self.which.set(Which::Adder);
            let total_balance = env.balance();
            let accumulator = accumulator::Accumulator::new(accumulator_code_hash, init_value)
                .value(total_balance / 4)
                .create()
                .expect("failed at instantiating the accumulator contract");
            self.accumulator.set(accumulator.clone());
            self.adder.set(
                adder::Adder::new(adder_code_hash, accumulator.account_id())
                    .value(total_balance / 4)
                    .create()
                    .expect("failed at instantiating the adder contract")
            );
            self.subber.set(
                subber::Subber::new(subber_code_hash, accumulator.account_id())
                    .value(total_balance / 4)
                    .create()
                    .expect("failed at instantiating the subber contract")
            );
        }
    }

    impl Delegator {
        /// Delegates the call.
        pub(external) fn delegate(&mut self, by: i32) {
            match &*self.which {
                Which::Adder => self.adder.inc(by),
                Which::Subber => self.subber.dec(by),
            }
        }

        /// Switches the delegator.
        pub(external) fn switch(&mut self) {
            match *self.which {
                Which::Adder => {
                    *self.which = Which::Subber;
                }
                Which::Subber => {
                    *self.which = Which::Adder;
                }
            }
        }
    }
}
