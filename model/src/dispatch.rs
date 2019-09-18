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
    FnInput,
    FnOutput,
    FnSelector,
    Message,
    Storage,
};
#[cfg(test)]
use core::any::TypeId;
use derive_more::Display;
use ink_core::memory::vec::Vec;
use scale::Decode;

/// A hash to identify a called function.
#[derive(Copy, Clone, PartialEq, Eq, Decode)]
pub struct Selector(u32);

impl Selector {
    /// Creates a new message handler selector from the given value.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }
}

/// The raw data with which a contract is being called.
pub struct CallAbi {
    /// The decoded message selector.
    selector: Selector,
    /// The raw undecoded parameter bytes.
    raw_params: Vec<u8>,
}

impl Decode for CallAbi {
    fn decode<I: scale::Input>(
        input: &mut I,
    ) -> core::result::Result<Self, scale::Error> {
        let selector = Selector::decode(input)?;
        let mut param_buf = Vec::new();
        while let Ok(byte) = input.read_byte() {
            param_buf.push(byte)
        }
        Ok(Self {
            selector,
            raw_params: param_buf,
        })
    }
}

impl CallAbi {
    /// Returns the message handler selector part of this call data.
    pub fn selector(&self) -> Selector {
        self.selector
    }

    /// Returns the actual call data in binary format.
    pub fn params(&self) -> &[u8] {
        self.raw_params.as_slice()
    }

    /// Creates a proper call data from a message and its required input.
    ///
    /// # Note
    ///
    /// This should normally only be needed in test code if a user
    /// wants to test the handling of a specific message.
    pub fn from_msg<Msg>(args: <Msg as FnInput>::Input) -> Self
    where
        Msg: Message,
        <Msg as FnInput>::Input: scale::Encode,
    {
        use scale::Encode;
        Self {
            selector: <Msg as FnSelector>::SELECTOR,
            raw_params: args.encode(),
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
            Msg: Message,
            S: Storage,
        {
            /// The dispatchable function.
            dispatchable: $dispatchable_fn<Msg, S>,
        }

        impl<Msg, S> FnSelector for $name<Msg, S>
        where
            Msg: Message,
            S: Storage,
        {
            const SELECTOR: Selector = <Msg as FnSelector>::SELECTOR;
        }

        impl<Msg, S> Copy for $name<Msg, S>
        where
            Msg: Message,
            S: Storage,
        {
        }

        impl<Msg, S> Clone for $name<Msg, S>
        where
            Msg: Message,
            S: Storage,
        {
            fn clone(&self) -> Self {
                Self {
                    dispatchable: self.dispatchable,
                }
            }
        }

        impl<Msg, S> $name<Msg, S>
        where
            Msg: Message,
            S: Storage,
        {
            /// Constructs a message handler from its raw counterpart.
            pub const fn new(dispatchable: $dispatchable_fn<Msg, S>) -> Self {
                Self { dispatchable }
            }

            /// Returns the associated handler selector.
            pub const fn selector() -> Selector {
                <Msg as FnSelector>::SELECTOR
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
            <Msg as FnOutput>::Output: scale::Encode,
            S: Storage,
        {
            fn dispatch(&self, storage: &mut S, data: CallAbi) -> Result<()> {
                let args = <Msg as FnInput>::Input::decode(&mut &data.params()[..])
                    .map_err(|_| Error::InvalidArguments)?;
                let result = self.eval(storage, args);
                if Msg::IS_MUT {
                    storage.flush();
                }
                fn is_unit_type<T: 'static>(_unit: &T) -> bool {
                    use core::any::TypeId;
                    TypeId::of::<()>() == TypeId::of::<T>()
                }
                if !is_unit_type(&result) {
                    // storage
                    //     .env()
                    //     .set_exec_output(&result)
                    //     .expect("this is the first time we set the output; qed");
                }
                Ok(())
            }
        }

        #[cfg(test)]
        impl<M, S> DispatchReturn<S> for $name<M, S>
        where
            M: Message + 'static,
            <M as FnInput>::Input: 'static,
            <M as FnOutput>::Output: 'static,
            S: Storage,
        {
            fn dispatch_return<M2>(&self, storage: &mut S, input: <M2 as FnInput>::Input) -> Result<<M2 as FnOutput>::Output>
            where
                M2: Message + 'static,
                <M2 as FnInput>::Input: 'static,
                <M2 as FnOutput>::Output: 'static,
            {
                if TypeId::of::<M>() != TypeId::of::<M2>() {
                    return Err(Error::InvalidArguments)
                }
                // M and M2 are equal at this point.
                // And with this follows:
                // - <M as FnInput>::Input == <M2 as FnInput>::Input
                // - <M as FnOutput>::Output == <M2 as FnOutput>::Output
                let input: <M as FnInput>::Input = unsafe {
                    core::mem::transmute_copy::<_, _>(&input)
                };
                let output: <M as FnOutput>::Output = self.eval(storage, input);
                let output: <M2 as FnOutput>::Output = unsafe {
                    core::mem::transmute_copy::<_, _>(&output)
                };
                Ok(output)
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
    ( // Forwarding rule for no `mut`
        $( #[$meta:meta] )*
        struct $name:ident( $dispatchable_fn:ident );
    ) => {
        impl_dispatcher_for! {
            $( #[$meta:meta] )*
            struct $name( $dispatchable_fn );
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

/// Errors the may occure during message dispatch.
#[derive(Debug, Display)]
pub enum Error {
    /// Encountered when the dispatcher was given an unknown function selector.
    #[display(fmt = "encountered unknown function selector")]
    InvalidFunctionSelector,
    /// Encountered when the arguments given to the dispatched function
    /// couldn't be decoded properly.
    #[display(fmt = "encountered invalid arguments for selected function")]
    InvalidArguments,
}

/// An error code returned to the SRML contract host side upon `create` and `call`.
#[derive(Debug, Copy, Clone)]
pub struct ErrCode(u8);

impl From<Error> for ErrCode {
    fn from(error: Error) -> Self {
        match error {
            Error::InvalidFunctionSelector => ErrCode::unknown_selector(),
            Error::InvalidArguments => ErrCode::invalid_arguments(),
        }
    }
}

impl ErrCode {
    /// Returns success.
    pub fn success() -> Self {
        ErrCode(0)
    }

    /// Unknown function selector.
    pub fn unknown_selector() -> Self {
        ErrCode(1)
    }

    /// Invalid arguments.
    pub fn invalid_arguments() -> Self {
        ErrCode(2)
    }

    /// Converts the error code into a `u32`.
    pub fn to_u32(self) -> u32 {
        self.0 as u32
    }
}

/// Results of message handling operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Types implementing this trait can handle contract calls.
pub trait Dispatch<S> {
    /// Dispatches the call and returns the encoded result.
    fn dispatch(&self, storage: &mut S, input: CallAbi) -> Result<()>;
}

/// Trait for off-chain test environments to test calls to concrete messages.
#[cfg(test)]
pub trait DispatchReturn<S>
where
    S: Storage,
{
    /// Dispatches into the given message `M` and returns the result or an appropriate error.
    fn dispatch_return<M>(
        &self,
        storage: &mut S,
        input: <M as FnInput>::Input,
    ) -> Result<<M as FnOutput>::Output>
    where
        M: Message + 'static,
        <M as FnInput>::Input: 'static,
        <M as FnOutput>::Output: 'static;
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
    fn dispatch(&self, _storage: &mut S, _data: CallAbi) -> Result<()> {
        Err(Error::InvalidFunctionSelector)
    }
}

#[cfg(test)]
impl<S> DispatchReturn<S> for UnreachableDispatcher
where
    S: Storage,
{
    fn dispatch_return<M>(
        &self,
        _storage: &mut S,
        _input: <M as FnInput>::Input,
    ) -> Result<<M as FnOutput>::Output>
    where
        M: Message + 'static,
        <M as FnInput>::Input: 'static,
        <M as FnOutput>::Output: 'static,
    {
        Err(Error::InvalidFunctionSelector)
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
    S: Storage,
    D: Dispatch<S> + FnSelector,
    Rest: Dispatch<S>,
{
    fn dispatch(&self, storage: &mut S, data: CallAbi) -> Result<()> {
        if <D as FnSelector>::SELECTOR == data.selector() {
            self.dispatcher.dispatch(storage, data)
        } else {
            self.rest.dispatch(storage, data)
        }
    }
}

#[cfg(test)]
impl<S, D, Rest> DispatchReturn<S> for DispatchList<D, Rest>
where
    S: Storage,
    D: DispatchReturn<S> + FnSelector,
    Rest: DispatchReturn<S>,
{
    fn dispatch_return<M>(
        &self,
        storage: &mut S,
        input: <M as FnInput>::Input,
    ) -> Result<<M as FnOutput>::Output>
    where
        M: Message + 'static,
        <M as FnInput>::Input: 'static,
        <M as FnOutput>::Output: 'static,
    {
        if <D as FnSelector>::SELECTOR == <M as FnSelector>::SELECTOR {
            self.dispatcher.dispatch_return::<M>(storage, input)
        } else {
            self.rest.dispatch_return::<M>(storage, input)
        }
    }
}
