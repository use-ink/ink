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

use crate::{
    CallAbi,
    Constructor,
    Dispatch,
    DispatchList,
    DispatchableFn,
    DispatchableFnMut,
    Dispatcher,
    DispatcherMut,
    EmptyDispatchList,
    Error,
    Message,
    PushDispatcher,
    Storage,
};
use core::marker::PhantomData;
use ink_core::storage::{
    alloc::{
        AllocateUsing,
        BumpAlloc,
    },
    Key,
};

/// A contract definition.
pub struct Contract<S, C, M> {
    pub(crate) storage: S,
    pub(crate) constructors: C,
    pub(crate) messages: M,
}

impl Contract<(), (), ()> {
    /// Creates a new contract definition for the given storage type.
    pub fn with_storage<S>() -> ContractBuilder<S, EmptyDispatchList, EmptyDispatchList> {
        ContractBuilder {
            storage: Default::default(),
            constructors: DispatchList::empty(),
            messages: DispatchList::empty(),
        }
    }
}

/// Storage marker.
#[derive(Debug, Copy, Clone)]
struct StorageMarker<S>(PhantomData<fn() -> S>);

impl<S> Default for StorageMarker<S> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Simplifies declaration of a smart contract.
pub struct ContractBuilder<S, C, M> {
    storage: StorageMarker<S>,
    constructors: C,
    messages: M,
}

impl<S, C> ContractBuilder<S, C, EmptyDispatchList>
where
    S: Storage,
    C: PushDispatcher,
{
    /// Pushes a new constructor to the contract definition.
    pub fn on_construct<C2>(
        self,
        dfn: DispatchableFnMut<C2, S>,
    ) -> ContractBuilder<S, DispatchList<DispatcherMut<C2, S>, C>, EmptyDispatchList>
    where
        S: Storage,
        C2: Constructor,
    {
        ContractBuilder {
            storage: self.storage,
            constructors: self.constructors.push(DispatcherMut::new(dfn)),
            messages: self.messages,
        }
    }
}

impl<S, M, C, RestC> ContractBuilder<S, DispatchList<C, RestC>, M>
where
    S: Storage,
    M: PushDispatcher,
{
    /// Pushes a new `&self` message to the contract definition.
    pub fn on_msg<M2>(
        self,
        dfn: DispatchableFn<M2, S>,
    ) -> ContractBuilder<S, DispatchList<C, RestC>, DispatchList<Dispatcher<M2, S>, M>>
    where
        S: Storage,
        M2: Message,
    {
        ContractBuilder {
            storage: self.storage,
            constructors: self.constructors,
            messages: self.messages.push(Dispatcher::new(dfn)),
        }
    }

    /// Pushes a new `&mut self` message to the contract definition.
    pub fn on_msg_mut<M2>(
        self,
        dfn: DispatchableFnMut<M2, S>,
    ) -> ContractBuilder<S, DispatchList<C, RestC>, DispatchList<DispatcherMut<M2, S>, M>>
    where
        S: Storage,
        M2: Message,
    {
        ContractBuilder {
            storage: self.storage,
            constructors: self.constructors,
            messages: self.messages.push(DispatcherMut::new(dfn)),
        }
    }
}

impl<S, C, RestC, M, RestM>
    ContractBuilder<S, DispatchList<C, RestC>, DispatchList<M, RestM>>
where
    S: Storage,
{
    /// Finalizes construction of the contract definition.
    pub fn done(self) -> Contract<S, DispatchList<C, RestC>, DispatchList<M, RestM>> {
        let storage: S = unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            AllocateUsing::allocate_using(&mut alloc)
        };
        Contract {
            storage,
            constructors: self.constructors,
            messages: self.messages,
        }
    }
}

/// Interface to work with instantiated contracts.
pub trait Instance {
    /// Constructs the contract from the given call data.
    fn construct(self, call_data: CallAbi) -> Result<(), Error>;
    /// Calls the contract from the given call data.
    fn call(self, call_data: CallAbi) -> Result<(), Error>;
}

impl<S, C, RestC, M, RestM> Instance
    for Contract<S, DispatchList<C, RestC>, DispatchList<M, RestM>>
where
    S: Storage,
    DispatchList<C, RestC>: Dispatch<S>,
    DispatchList<M, RestM>: Dispatch<S>,
{
    fn construct(self, call_data: CallAbi) -> Result<(), Error> {
        let mut this = self;
        this.constructors.dispatch(&mut this.storage, call_data)
    }

    fn call(self, call_data: CallAbi) -> Result<(), Error> {
        let mut this = self;
        this.messages.dispatch(&mut this.storage, call_data)
    }
}
