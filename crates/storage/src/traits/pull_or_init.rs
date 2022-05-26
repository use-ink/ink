// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use crate::traits::{
    pull_storage,
    storage::OnCallInitializer,
    DecodeWrapper,
    Storable,
};
use ink_primitives::Key;

pub struct PullOrInit<T: Storable> {
    marker: core::marker::PhantomData<fn() -> T>,
}

impl<T: OnCallInitializer + Storable> PullOrInit<T> {
    #[allow(dead_code)]
    pub fn pull_or_init(key: &Key) -> T {
        let maybe_instance =
            ink_env::get_contract_storage::<(), DecodeWrapper<T>>(key, None);
        match maybe_instance {
            Ok(None) | Err(_) => {
                let mut instance = Default::default();
                <T as OnCallInitializer>::initialize(&mut instance);
                instance
            }
            Ok(Some(wrapper)) => wrapper.0,
        }
    }
}

pub trait PullOrInitFallback<T: Storable> {
    #[allow(dead_code)]
    fn pull_or_init(key: &Key) -> T {
        pull_storage(key)
    }
}
impl<T: Storable> PullOrInitFallback<T> for PullOrInit<T> {}

/// Pulls the struct from the storage or creates and new one and inits it.
#[macro_export]
#[doc(hidden)]
macro_rules! pull_or_init {
    ( $T:ty, $key:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::traits::pull_or_init::PullOrInitFallback as _;

        $crate::traits::pull_or_init::PullOrInit::<$T>::pull_or_init(&$key)
    }};
}

#[cfg(test)]
mod tests {
    use crate::traits::{
        push_storage,
        OnCallInitializer,
    };
    use ink_primitives::Key;

    #[derive(Default, scale::Encode, scale::Decode)]
    struct U32(u32);

    impl OnCallInitializer for U32 {
        fn initialize(&mut self) {
            self.0 = 123;
        }
    }

    #[ink_lang::test]
    fn init_works() {
        const key: Key = 111;
        let instance = pull_or_init!(U32, key);
        assert_eq!(123, instance.0);
    }

    #[ink_lang::test]
    fn pull_or_init_works() {
        const KEY: Key = 111;
        push_storage(&U32(456), &KEY);
        let instance = pull_or_init!(U32, KEY);

        // Instead of init we used a pulled value
        assert_eq!(456, instance.0);
    }

    #[ink_lang::test]
    #[should_panic(expected = "storage entry was empty")]
    fn pull_or_init_fails() {
        const key: Key = 111;
        let instance = pull_or_init!(u32, key);
        assert_eq!(123, instance);
    }

    #[ink_lang::test]
    fn pull_works() {
        const key: Key = 111;
        push_storage(&321, &key);
        let instance = pull_or_init!(u32, key);
        assert_eq!(321, instance);
    }
}
