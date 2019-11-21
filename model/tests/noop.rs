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

#![allow(dead_code)]

use ink_core::env::{
    ContractEnv,
    DefaultSrmlTypes,
};
use ink_model::{
    messages,
    state,
    Contract,
    ContractDecl,
    EnvHandler,
};

state! {
    /// The simplest contract that can still be deployed and called.
    struct Noop {}
}

messages! {
    /// Multiline comment
    /// for a function
    /// which does nothing.
    [0u8; 4] => DoNothing();
}

impl Noop {
    pub fn deploy(&mut self, _env: &mut EnvHandler<ContractEnv<DefaultSrmlTypes>>) {}
    pub fn do_nothing(&self, _env: &EnvHandler<ContractEnv<DefaultSrmlTypes>>) {}
}

#[rustfmt::skip]
fn instantiate() -> impl Contract {
	ContractDecl::using::<Noop, ContractEnv<DefaultSrmlTypes>>()
		.on_deploy(|env, ()| {
            let (handler, state) = env.split_mut();
            state.deploy(handler)
        })
        .on_msg::<DoNothing>(|env, ()| {
            let (handler, state) = env.split();
            state.do_nothing(handler)
        })
		.instantiate()
}
