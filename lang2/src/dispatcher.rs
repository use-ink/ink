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
    AccessEnv,
    DispatchError,
    FnInput,
    FnOutput,
    FnSelector,
    Message,
};
use core::any::TypeId;
use ink_core::{
    env2::call::{
        CallData,
        Selector,
    },
    storage::Flush,
};

/// Results of message handling operations.
pub type Result<T> = core::result::Result<T, DispatchError>;

/// Types implementing this trait can handle contract calls.
pub trait Dispatch<S> {
    /// Dispatches the call and returns the call result.
    fn dispatch<Env>(&self, storage: &mut S, input: &CallData) -> Result<()>
    where
        S: AccessEnv<Env>,
        Env: ink_core::env2::Env;
}

/// A dispatcher that shall never dispatch.
///
/// # Note
///
/// This always comes last in a chain of dispatchers
/// and is used to break out of the dispatch routine.
#[derive(Copy, Clone)]
pub struct UnreachableDispatcher;

impl<S> Dispatch<S> for UnreachableDispatcher {
    fn dispatch<Env>(&self, _storage: &mut S, _data: &CallData) -> Result<()>
    where
        S: AccessEnv<Env>,
        Env: ink_core::env2::Env,
    {
        Err(DispatchError::UnknownSelector)
    }
}

/// Types able to push another dispatcher to themselves.
pub trait PushDispatcher: Sized {
    fn push<D>(self, dispatcher: D) -> DispatchList<D, Self>;
}

impl PushDispatcher for UnreachableDispatcher {
    /// Creates a dispatch list with `dispatcher` and `self` as elements.
    fn push<D>(self, dispatcher: D) -> DispatchList<D, UnreachableDispatcher> {
        DispatchList {
            dispatcher,
            rest: self,
        }
    }
}

/// A simple type definition to view the single unreachable dispatcher as list.
pub type EmptyDispatchList = UnreachableDispatcher;

/// A list of dispatchers.
pub struct DispatchList<D, Rest> {
    /// The current dispatcher.
    dispatcher: D,
    /// The rest of the dispatchers.
    rest: Rest,
}

impl DispatchList<(), ()> {
    /// Creates a new dispatch list.
    pub fn empty() -> UnreachableDispatcher {
        UnreachableDispatcher
    }
}

impl<D, Rest> PushDispatcher for DispatchList<D, Rest> {
    /// Pushes another dispatcher onto the list.
    fn push<D2>(self, dispatcher: D2) -> DispatchList<D2, Self> {
        DispatchList {
            dispatcher,
            rest: self,
        }
    }
}

impl<S, D, Rest> Dispatch<S> for DispatchList<D, Rest>
where
    D: Dispatch<S> + FnSelector,
    Rest: Dispatch<S>,
{
    fn dispatch<Env>(&self, storage: &mut S, data: &CallData) -> Result<()>
    where
        S: AccessEnv<Env>,
        Env: ink_core::env2::Env,
    {
        if <D as FnSelector>::SELECTOR == data.selector() {
            self.dispatcher.dispatch(storage, data)
        } else {
            self.rest.dispatch(storage, data)
        }
    }
}

/// A function with the signature able to handle
/// storage preserving calls of messages or constructors.
pub type DispatchableFn<Msg, S> =
    fn(&S, <Msg as FnInput>::Input) -> <Msg as FnOutput>::Output;

/// A function with the signature able to handle potentially
/// storage mutating calls of messages or constructors.
pub type DispatchableFnMut<Msg, S> =
    fn(&mut S, <Msg as FnInput>::Input) -> <Msg as FnOutput>::Output;

macro_rules! impl_dispatcher_for {
    (
        $( #[$meta:meta] )*
        struct $name:ident( $dispatchable_fn:ident ); $($tt:tt)?
    ) => {
        $( #[$meta] )*
        pub struct $name<Msg, S>
        where
            Msg: FnInput + FnOutput,
        {
            /// The dispatchable function.
            dispatchable: $dispatchable_fn<Msg, S>,
        }

        impl<Msg, S> FnSelector for $name<Msg, S>
        where
            Msg: FnInput + FnOutput + FnSelector,
        {
            const SELECTOR: Selector = <Msg as FnSelector>::SELECTOR;
        }

        impl<Msg, S> Copy for $name<Msg, S>
        where
            Msg: FnInput + FnOutput,
        {
        }

        impl<Msg, S> Clone for $name<Msg, S>
        where
            Msg: FnInput + FnOutput,
        {
            fn clone(&self) -> Self {
                Self {
                    dispatchable: self.dispatchable,
                }
            }
        }

        impl<Msg, S> $name<Msg, S>
        where
            Msg: FnInput + FnOutput + FnSelector,
        {
            /// Returns the associated handler selector.
            pub const fn selector() -> Selector {
                <Msg as FnSelector>::SELECTOR
            }
        }

        impl<Msg, S> $name<Msg, S>
        where
            Msg: FnInput + FnOutput,
        {
            /// Constructs a message handler from its raw counterpart.
            pub const fn new(dispatchable: $dispatchable_fn<Msg, S>) -> Self {
                Self { dispatchable }
            }

            /// Calls the dispatchable function and returns its result.
            pub fn eval(
                &self,
                storage: & $($tt)* S,
                inputs: <Msg as FnInput>::Input,
            ) -> <Msg as FnOutput>::Output {
                (self.dispatchable)(storage, inputs)
            }
        }

        impl<Msg, S> Dispatch<S> for $name<Msg, S>
        where
            Msg: Message,
            <Msg as FnInput>::Input: scale::Decode,
            <Msg as FnOutput>::Output: scale::Encode,
            S: Flush,
        {
            fn dispatch<Env>(&self, storage: &mut S, data: &CallData) -> Result<()>
            where
                S: AccessEnv<Env>,
                Env: ink_core::env2::Env,
            {
                use scale::Decode as _;
                let args = <Msg as FnInput>::Input::decode(&mut &data.params()[..])
                    .map_err(|_| DispatchError::InvalidParameters)?;
                let result = self.eval(storage, args);
                if TypeId::of::<<Msg as FnOutput>::Output>() != TypeId::of::<()>() {
                    AccessEnv::access_env(storage).output(&result)
                }
                if <Msg as Message>::IS_MUT {
                    // Flush the storage since the message might have mutated it.
                    Flush::flush(storage);
                }
                Ok(())
            }
        }
    };
    ( // Forwarding rule for `mut`
        $( #[$meta:meta] )*
        struct $name:ident( mut $dispatchable_fn:ident );
    ) => {
        impl_dispatcher_for! {
            $( #[$meta] )*
            struct $name( $dispatchable_fn ); mut
        }
    };
}

impl_dispatcher_for! {
    /// Dispatcher for storage preserving messages.
    struct Dispatcher(DispatchableFn);
}

impl_dispatcher_for! {
    /// Dispatcher for potentially storage mutating messages and constructors.
    struct DispatcherMut(mut DispatchableFnMut);
}
