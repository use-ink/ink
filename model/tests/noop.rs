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
    constructors,
    messages,
    storage,
    Contract,
    Instance,
};

storage! {
    /// The simplest contract that can still be deployed and called.
    struct Noop {}
}

constructors! { 0 => ConstructNothing(); }
messages! { 0 => DoNothing(&self); }

#[rustfmt::skip]
fn declare() -> impl Instance {
	Contract::with_storage::<Noop<ContractEnv<DefaultSrmlTypes>>>()
		.on_construct::<ConstructNothing>(|_contract, _| {
            ()
        })
        .on_msg::<DoNothing>(|_contract, _| {
            ()
        })
		.done()
}
