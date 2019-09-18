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
    Constructor,
    Contract,
    DispatchList,
    DispatchReturn,
    FnInput,
    FnOutput,
    Message,
    Storage,
};
use derive_more::From;

/// Trait implemented by contract declarations to create testable instances of them.
pub trait TestConstructInstance: Sized {
    /// Calls the provided constructor to initialize the contract with.
    ///
    /// # Panics
    ///
    /// If the provided constructor is unknown to the contract.
    fn construct_with<C>(self, inputs: <C as FnInput>::Input) -> TestableContract<Self>
    where
        C: Constructor + 'static,
        <C as FnInput>::Input: 'static,
        <C as FnOutput>::Output: 'static;
}

impl<S, C, RestC, M, RestM> TestConstructInstance
    for Contract<S, DispatchList<C, RestC>, DispatchList<M, RestM>>
where
    S: Storage,
    C: Constructor,
    <C as FnInput>::Input: 'static,
    M: Message,
    DispatchList<C, RestC>: DispatchReturn<S>,
{
    fn construct_with<C2>(self, inputs: <C2 as FnInput>::Input) -> TestableContract<Self>
    where
        C2: Constructor + 'static,
        <C2 as FnInput>::Input: 'static,
        <C2 as FnOutput>::Output: 'static,
    {
        let mut this = self;
        // TODO: Why do we need the `let _ = ..;` here? (Warnings!)
        let _ = this
            .constructors
            .dispatch_return::<C2>(&mut this.storage, inputs)
            .expect("failed at constructing the testable contract");
        this.into()
    }
}

/// A testable contract instance.
///
/// Restricts the interface to provide only contract messages after construction.
#[derive(From)]
pub struct TestableContract<C> {
    /// The wrapped smart contract.
    contract: C,
}

/// Trait implemented by testable contract instances.
pub trait TestCallInstance {
    /// Calls the provided message of the contract.
    ///
    /// # Panics
    ///
    /// If the provided message is unknown to the contract.
    fn call_mut<M>(&mut self, input: <M as FnInput>::Input) -> <M as FnOutput>::Output
    where
        M: Message + 'static,
        <M as FnInput>::Input: scale::Encode + 'static,
        <M as FnOutput>::Output: scale::Decode + 'static;
}

impl<S, C, RestC, M, RestM> TestCallInstance
    for TestableContract<Contract<S, DispatchList<C, RestC>, DispatchList<M, RestM>>>
where
    S: Storage,
    C: Constructor,
    <C as FnInput>::Input: 'static,
    M: Message,
    DispatchList<M, RestM>: DispatchReturn<S>,
{
    fn call_mut<M2>(&mut self, input: <M2 as FnInput>::Input) -> <M2 as FnOutput>::Output
    where
        M2: Message + 'static,
        <M2 as FnInput>::Input: scale::Encode + 'static,
        <M2 as FnOutput>::Output: scale::Decode + 'static,
    {
        // TODO: Why do we need the `let _ = ..;` here? (Warnings!)
        self.contract
            .messages
            .dispatch_return::<M2>(&mut self.contract.storage, input)
            .expect("failed at evaluating a message of a testable contract")
    }
}
