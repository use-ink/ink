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

use ink_core::storage;
use ink_lang::contract;

contract! {
    /// Tests emitting of custom defined events.
    struct CallCounter {
        /// A simple counter for the calls.
        count: storage::Value<u32>,
    }

    impl Deploy for CallCounter {
        fn deploy(&mut self) {
            self.count.set(0)
        }
    }

    event IncCalled { current: u32 }
    event DecCalled { current: u32 }

    impl CallCounter {
        /// Increments the internal counter.
        ///
        /// # Note
        ///
        /// Also emits an event.
        pub(external) fn inc(&mut self) {
            self.count += 1;
            env.emit(IncCalled { current: *self.count });
        }

        /// Decrements the internal counter.
        ///
        /// # Note
        ///
        /// Also emits an event.
        pub(external) fn dec(&mut self) {
            self.count -= 1;
            env.emit(DecCalled { current: *self.count });
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;

    #[test]
    fn it_works() {
        let mut contract = CallCounter::deploy_mock();
        assert_eq!(env::test::emitted_events().count(), 0);
        contract.inc();
        assert_eq!(env::test::emitted_events().count(), 1);
        contract.dec();
        assert_eq!(env::test::emitted_events().count(), 2);
    }
}
