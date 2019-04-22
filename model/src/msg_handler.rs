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
    exec_env::ExecutionEnv,
    msg::Message,
    state::ContractState,
};
use core::{
    marker::PhantomData,
    result::Result as CoreResult,
};
use parity_codec::Decode;
use pdsl_core::memory::vec::Vec;

/// A raw read-only message handler for the given message and state.
///
/// # Note
///
/// - Read-only message handlers cannot mutate contract state.
/// - Requires `Msg` to impl `Message` and `State` to impl `ContractState`.
pub type RawMessageHandler<Msg, State> =
    fn(&ExecutionEnv<State>, <Msg as Message>::Input) -> <Msg as Message>::Output;

/// A raw mutable message handler for the given message and state.
///
/// # Note
///
/// - Mutable message handlers may mutate contract state.
/// - Requires `Msg` to impl `Message` and `State` to impl `ContractState`.
pub type RawMessageHandlerMut<Msg, State> =
    fn(&mut ExecutionEnv<State>, <Msg as Message>::Input) -> <Msg as Message>::Output;

/// The raw data with which a contract is being called.
pub struct CallData {
    /// The decoded message selector.
    selector: MessageHandlerSelector,
    /// The raw undecoded parameter bytes.
    raw_params: Vec<u8>,
}

impl Decode for CallData {
    fn decode<I: parity_codec::Input>(input: &mut I) -> Option<Self> {
        let selector = MessageHandlerSelector::decode(input)?;
        let mut param_buf = Vec::new();
        while let Some(byte) = input.read_byte() {
            param_buf.push(byte)
        }
        Some(Self {
            selector,
            raw_params: param_buf,
        })
    }
}

impl CallData {
    /// Returns the message handler selector part of this call data.
    pub fn selector(&self) -> MessageHandlerSelector {
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
    pub fn from_msg<Msg>(args: <Msg as Message>::Input) -> Self
    where
        Msg: Message,
        <Msg as Message>::Input: parity_codec::Encode,
    {
        use parity_codec::Encode;
        Self {
            selector: <Msg as Message>::ID,
            raw_params: args.encode(),
        }
    }
}

/// A hash to identify a called function.
#[derive(Copy, Clone, PartialEq, Eq, Decode)]
pub struct MessageHandlerSelector(u32);

impl MessageHandlerSelector {
    /// Creates a new message handler selector from the given value.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }
}

/// A read-only message handler.
///
/// Read-only message handlers cannot mutate contract state.
pub struct MessageHandler<Msg, State>
where
    Msg: Message,
    State: ContractState,
{
    /// Required in order to trick Rust into thinking that it actually owns a message.
    ///
    /// However, in general message types are zero-sized-types (ZST).
    msg_marker: PhantomData<Msg>,
    /// The actual mutable handler for the message and state.
    raw_handler: RawMessageHandler<Msg, State>,
}

impl<Msg, State> MessageHandler<Msg, State>
where
    Msg: Message,
    State: ContractState,
{
    /// Returns the associated handler selector.
    pub const fn selector() -> MessageHandlerSelector {
        <Msg as Message>::ID
    }
}

impl<Msg, State> Copy for MessageHandler<Msg, State>
where
    Msg: Message,
    State: ContractState,
{
}

impl<Msg, State> Clone for MessageHandler<Msg, State>
where
    Msg: Message,
    State: ContractState,
{
    fn clone(&self) -> Self {
        Self {
            msg_marker: self.msg_marker,
            raw_handler: self.raw_handler,
        }
    }
}

impl<Msg, State> MessageHandler<Msg, State>
where
    Msg: Message,
    State: ContractState,
{
    /// Constructs a message handler from its raw counterpart.
    pub const fn from_raw(raw_handler: RawMessageHandler<Msg, State>) -> Self {
        Self {
            msg_marker: PhantomData,
            raw_handler,
        }
    }
}

/// A mutable message handler.
///
/// Mutable message handlers may mutate contract state.
///
/// # Note
///
/// This is a thin wrapper around a raw message handler in order
/// to provide more type safety and better interfaces.
pub struct MessageHandlerMut<Msg, State>
where
    Msg: Message,
    State: ContractState,
{
    /// Required in order to trick Rust into thinking that it actually owns a message.
    ///
    /// However, in general message types are zero-sized-types (ZST).
    msg_marker: PhantomData<Msg>,
    /// The actual read-only handler for the message and state.
    raw_handler: RawMessageHandlerMut<Msg, State>,
}

impl<Msg, State> Copy for MessageHandlerMut<Msg, State>
where
    Msg: Message,
    State: ContractState,
{
}

impl<Msg, State> Clone for MessageHandlerMut<Msg, State>
where
    Msg: Message,
    State: ContractState,
{
    fn clone(&self) -> Self {
        Self {
            msg_marker: self.msg_marker,
            raw_handler: self.raw_handler,
        }
    }
}

impl<Msg, State> MessageHandlerMut<Msg, State>
where
    Msg: Message,
    State: ContractState,
{
    /// Constructs a message handler from its raw counterpart.
    pub const fn from_raw(raw_handler: RawMessageHandlerMut<Msg, State>) -> Self {
        Self {
            msg_marker: PhantomData,
            raw_handler,
        }
    }
}

impl<Msg, State> MessageHandlerMut<Msg, State>
where
    Msg: Message,
    State: ContractState,
{
    /// Returns the associated handler selector.
    pub const fn selector() -> MessageHandlerSelector {
        <Msg as Message>::ID
    }
}

/// Errors the may occure during message handling.
pub enum Error {
    /// Encountered when no function selector
    /// matched the given input bytes representing
    /// the function selector.
    InvalidFunctionSelector,
    /// Encountered when wrong parameters have
    /// been given to a selected function.
    InvalidArguments,
}

impl Error {
    /// Returns a short description of the error.
    pub fn description(&self) -> &'static str {
        match self {
            Error::InvalidFunctionSelector => "encountered invalid message selector",
            Error::InvalidArguments => {
                "encountered invalid arguments for selected message"
            }
        }
    }
}

/// Results of message handling operations.
pub type Result<T> = CoreResult<T, Error>;

/// Types implementing this trait can handle contract calls.
pub trait HandleCall<State> {
    /// Handles the call and returns the encoded result.
    fn handle_call(
        &self,
        env: &mut ExecutionEnv<State>,
        data: CallData,
    ) -> Result<Vec<u8>>;
}

/// A message handler that shall never handle a message.
///
/// # Note
///
/// Since this always comes last in a chain of message
/// handlers it can be used to check for incoming unknown
/// message selectors in call datas from the outside.
#[derive(Copy, Clone)]
pub struct UnreachableMessageHandler;

impl<State> HandleCall<State> for UnreachableMessageHandler {
    fn handle_call(
        &self,
        _env: &mut ExecutionEnv<State>,
        _data: CallData,
    ) -> Result<Vec<u8>> {
        Err(Error::InvalidFunctionSelector)
    }
}

macro_rules! impl_handle_call_for_chain {
    ( $msg_handler_kind:ident, requires_flushing: $requires_flushing:literal ) => {
        impl<Msg, State> HandleCall<State> for $msg_handler_kind<Msg, State>
        where
            Msg: Message,
            <Msg as Message>::Output: parity_codec::Encode,
            State: ContractState,
        {
            fn handle_call(
                &self,
                env: &mut ExecutionEnv<State>,
                data: CallData,
            ) -> Result<Vec<u8>> {
                let args = <Msg as Message>::Input::decode(&mut &data.params()[..])
                    .ok_or(Error::InvalidArguments)?;
                let result = (self.raw_handler)(env, args);
                if $requires_flushing {
                    env.state.flush()
                }
                use parity_codec::Encode;
                Ok(result.encode())
            }
        }

        impl<Msg, State, Rest> HandleCall<State> for ($msg_handler_kind<Msg, State>, Rest)
        where
            Msg: Message,
            <Msg as Message>::Output: 'static,
            State: ContractState,
            Rest: HandleCall<State>,
        {
            fn handle_call(
                &self,
                env: &mut ExecutionEnv<State>,
                data: CallData,
            ) -> Result<Vec<u8>> {
                let (handler, rest) = self;
                if $msg_handler_kind::<Msg, State>::selector() == data.selector() {
                    handler.handle_call(env, data)
                } else {
                    rest.handle_call(env, data)
                }
            }
        }
    };
}

impl_handle_call_for_chain!(MessageHandler, requires_flushing: false);
impl_handle_call_for_chain!(MessageHandlerMut, requires_flushing: true);
