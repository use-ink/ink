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
    storage: S,
    constructors: C,
    messages: M,
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

// /// An interface that allows for simple testing of contracts.
// pub trait TestableContract {
//     /// The arguments used for deployment.
//     ///
//     /// These must be the same as the ones defined on the deploy handler
//     /// of a contract declaration.
//     type DeployArgs: scale::Encode;

//     /// Deploys the contract given the provided arguments for deployment.
//     ///
//     /// # Note
//     ///
//     /// This shall be performed only once during the lifetime of a contract.
//     ///
//     /// # Panics
//     ///
//     /// This might panic if the provided arguments do not match the expected.
//     fn deploy(&mut self, deploy_args: Self::DeployArgs);

//     /// Calls the contract with the given message and its
//     /// inputs and upon successful execution returns its result.
//     ///
//     /// # Note
//     ///
//     /// Takes `&mut self` since it could potentially call a message
//     /// that mutates state. There currently is no separation between
//     /// messages that mutate state and those that do not.
//     ///
//     /// # Panics
//     ///
//     /// If the contract has no message handler setup for the given message.
//     fn call<Msg>(&mut self, input: <Msg as FnInput>::Input) -> <Msg as FnOutput>::Output
//     where
//         Msg: Message,
//         <Msg as FnInput>::Input: scale::Encode,
//         <Msg as FnOutput>::Output: scale::Decode;
// }

// impl<S, Env, DeployArgs, HandlerChain> TestableContract
//     for ContractInstance<S, Env, DeployArgs, HandlerChain>
// where
//     S: Storage,
//     Env: env::Env,
//     DeployArgs: scale::Codec,
//     HandlerChain: Dispatch<S>,
// {
//     type DeployArgs = DeployArgs;

//     fn deploy(&mut self, input: Self::DeployArgs) {
//         self.deploy_with(&input.encode()[..])
//             .expect("`deploy` failed to execute properly")
//     }

//     fn call<Msg>(&mut self, input: <Msg as FnInput>::Input) -> <Msg as FnOutput>::Output
//     where
//         Msg: Message,
//         <Msg as FnInput>::Input: scale::Encode,
//         <Msg as FnOutput>::Output: scale::Decode,
//     {
//         let encoded_result = self
//             .call_with(CallAbi::from_msg::<Msg>(input))
//             .expect("`call` failed to execute properly");
//         use scale::Decode;
//         <Msg as FnOutput>::Output::decode(&mut &encoded_result[..])
//             .expect("`call_with` only encodes the correct types")
//     }
// }
