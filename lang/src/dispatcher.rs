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

use crate::{
    DispatchError,
    FnInput,
    FnOutput,
    FnSelector,
    Message,
};
use core::any::TypeId;
use ink_core::{
    env::{
        call::{
            CallData,
            Selector,
        },
        EnvTypes,
    },
    storage::Flush,
};

/// Results of message handling operations.
pub type Result<T> = core::result::Result<T, DispatchError>;

/// Types implementing this trait can handle contract calls.
pub trait Dispatch<S> {
    /// Dispatches the call and returns the call result.
    fn dispatch<T>(&self, storage: &mut S, input: &CallData) -> Result<()>
    where
        T: EnvTypes;
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
    fn dispatch<T>(&self, _storage: &mut S, _data: &CallData) -> Result<()>
    where
        T: EnvTypes,
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
    fn dispatch<T>(&self, storage: &mut S, data: &CallData) -> Result<()>
    where
        T: EnvTypes,
    {
        if <D as FnSelector>::SELECTOR == data.selector() {
            self.dispatcher.dispatch::<T>(storage, data)
        } else {
            self.rest.dispatch::<T>(storage, data)
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
            fn dispatch<T>(&self, storage: &mut S, data: &CallData) -> Result<()>
            where
                T: EnvTypes,
            {
                use scale::Decode as _;
                let args = <Msg as FnInput>::Input::decode(&mut &data.params()[..])
                    .map_err(|_| DispatchError::InvalidParameters)?;
                let result = self.eval(storage, args);
                if TypeId::of::<<Msg as FnOutput>::Output>() != TypeId::of::<()>() {
                    ink_core::env::output::<<Msg as FnOutput>::Output>(&result)
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
