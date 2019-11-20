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

use ink_core::storage;
use ink_core::env::{
    ContractEnv,
    DefaultSrmlTypes,
};
use ink_lang::contract;

contract! {
    #![env = DefaultSrmlTypes]

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
            self.dec_internal(env);
        }
    }

    impl CallCounter {
        fn dec_internal(&mut self, env: &mut ink_model::EnvHandler<ContractEnv<DefaultSrmlTypes>>) {
            self.count -= 1;
            env.emit(DecCalled { current: *self.count });
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use ink_core::env;

    use super::*;

    #[test]
    fn it_works() {
        let mut contract = CallCounter::deploy_mock();
        assert_eq!(env::test::emitted_events::<DefaultSrmlTypes>().count(), 0);
        contract.inc();
        assert_eq!(env::test::emitted_events::<DefaultSrmlTypes>().count(), 1);
        contract.dec();
        assert_eq!(env::test::emitted_events::<DefaultSrmlTypes>().count(), 2);
    }
}
