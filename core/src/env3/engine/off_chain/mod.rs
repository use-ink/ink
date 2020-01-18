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

mod impls;

use super::Instance;
use core::cell::RefCell;

pub enum Accessor {}

pub struct TestEnv {}

impl Instance for Accessor {
    type Engine = TestEnv;

    fn run<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self::Engine) -> R,
    {
        thread_local!(
            static INSTANCE: RefCell<TestEnv> = RefCell::new(
                TestEnv {}
            )
        );
        INSTANCE.with(|instance| {
            f(&mut instance.borrow_mut())
        })
    }
}
