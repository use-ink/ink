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
};
use ink_primitives::StorageKey;
use scale::Decode;

pub struct PullOrInit<T: Decode> {
    marker: core::marker::PhantomData<fn() -> T>,
}

impl<T: OnCallInitializer + Decode + Sized> PullOrInit<T> {
    #[allow(dead_code)]
    pub fn pull_or_init(key: &StorageKey) -> T {
        let maybe_instance = ink_env::get_storage_value(key);
        if maybe_instance.is_err() || maybe_instance.as_ref().unwrap().is_none() {
            let mut instance = Default::default();
            <T as OnCallInitializer>::initialize(&mut instance);
            instance
        } else {
            maybe_instance.unwrap().unwrap()
        }
    }
}

pub trait PullOrInitFallback<T: Decode> {
    #[allow(dead_code)]
    fn pull_or_init(key: &StorageKey) -> T {
        pull_storage(key)
    }
}
impl<T: Decode> PullOrInitFallback<T> for PullOrInit<T> {}

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
    use ink_primitives::StorageKey;

    #[derive(Default, scale::Encode, scale::Decode)]
    struct U32(u32);

    impl OnCallInitializer for U32 {
        fn initialize(&mut self) {
            self.0 = 123;
        }
    }

    #[ink_lang::test]
    fn init_works() {
        const STORAGE_KEY: StorageKey = 111;
        let instance = pull_or_init!(U32, STORAGE_KEY);
        assert_eq!(123, instance.0);
    }

    #[ink_lang::test]
    fn pull_or_init_works() {
        const STORAGE_KEY: StorageKey = 111;
        push_storage(&U32(456), &STORAGE_KEY);
        let instance = pull_or_init!(U32, STORAGE_KEY);

        // Instead of init we used a pulled value
        assert_eq!(456, instance.0);
    }

    #[ink_lang::test]
    #[should_panic(expected = "storage entry was empty")]
    fn pull_or_init_fails() {
        const STORAGE_KEY: StorageKey = 111;
        let instance = pull_or_init!(u32, STORAGE_KEY);
        assert_eq!(123, instance);
    }

    #[ink_lang::test]
    fn pull_works() {
        const STORAGE_KEY: StorageKey = 111;
        push_storage(&321, &STORAGE_KEY);
        let instance = pull_or_init!(u32, STORAGE_KEY);
        assert_eq!(321, instance);
    }
}
