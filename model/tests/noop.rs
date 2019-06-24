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
    0 => DoNothing();
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
