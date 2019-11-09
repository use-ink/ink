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

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)]

use ink_core::storage;
use ink_lang2 as ink;

use accumulator::Accumulator;
use adder::Adder;
use subber::Subber;

#[ink::contract(version = "0.1.0")]
mod delegator {
    /// Specifies the state of the delegator.
    ///
    /// In `Adder` state the delegator will delegate to the `Adder` contract
    /// and in `Subber` state will delegate to the `Subber` contract.
    ///
    /// The initial state is `Adder`.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    pub enum Which {
        Adder,
        Subber,
    }

    impl ink_core::storage::Flush for Which {
        fn flush(&mut self) {}
    }

    /// Delegates calls to an adder or subber contract to mutate
    /// a value in an accumulator contract.
    ///
    /// In order to deploy the delegator smart contract we first
    /// have to manually put the code of the accumulator, adder
    /// and subber smart contracts, receive their code hashes from
    /// the signalled events and put their code hash into our
    /// delegator smart contract.
    #[ink(storage)]
    struct Delegator {
        /// Says which of adder or subber is currently in use.
        which: storage::Value<Which>,
        /// The accumulator smart contract.
        accumulator: storage::Value<Accumulator>,
        /// The adder smart contract.
        adder: storage::Value<Adder>,
        /// The subber smart contract.
        subber: storage::Value<Subber>,
    }

    impl Delegator {
        /// Instantiate a delegator with the given sub-contract codes.
        #[ink(constructor)]
        fn new(
            &mut self,
            init_value: i32,
            accumulator_code_hash: Hash,
            adder_code_hash: Hash,
            subber_code_hash: Hash,
        ) {
            self.which.set(Which::Adder);
            let total_balance = self.env().balance();
            let accumulator = Accumulator::new(init_value)
                .value(total_balance / 4)
                .using_code(accumulator_code_hash)
                .create_using(self.env())
                .expect("failed at instantiating the `Accumulator` contract");
            let adder = Adder::new(accumulator.clone())
                .value(total_balance / 4)
                .using_code(adder_code_hash)
                .create_using(self.env())
                .expect("failed at instantiating the `Adder` contract");
            let subber = Subber::new(accumulator.clone())
                .value(total_balance / 4)
                .using_code(subber_code_hash)
                .create_using(self.env())
                .expect("failed at instantiating the `Subber` contract");
            self.accumulator.set(accumulator);
            self.adder.set(adder);
            self.subber.set(subber);
        }

        /// Returns the accumulator's value.
        #[ink(message)]
        fn get(&self) -> i32 {
            self.accumulator.get().get()
        }

        /// Delegates the call to either `Adder` or `Subber`.
        #[ink(message)]
        fn change(&mut self, by: i32) {
            match &*self.which {
                Which::Adder => self.adder.inc(by),
                Which::Subber => self.subber.dec(by),
            }
        }

        /// Switches the delegator.
        #[ink(message)]
        fn switch(&mut self) {
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
