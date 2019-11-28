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

use core::{
    marker::PhantomData,
    result::Result as CoreResult,
};

use ink_core::env;
use ink_prelude::vec::Vec;
use scale::Decode;

use crate::{
    exec_env::ExecutionEnv,
    msg::Message,
    state::ContractState,
};

/// A raw read-only message handler for the given message and state.
///
/// # Note
///
/// - Read-only message handlers cannot mutate contract state.
/// - Requires `Msg` to impl `Message` and `State` to impl `ContractState`.
pub type RawMessageHandler<Msg, State, Env> =
    fn(&ExecutionEnv<State, Env>, <Msg as Message>::Input) -> <Msg as Message>::Output;

/// A raw mutable message handler for the given message and state.
///
/// # Note
///
/// - Mutable message handlers may mutate contract state.
/// - Requires `Msg` to impl `Message` and `State` to impl `ContractState`.
pub type RawMessageHandlerMut<Msg, State, Env> = fn(
    &mut ExecutionEnv<State, Env>,
    <Msg as Message>::Input,
) -> <Msg as Message>::Output;

/// The raw data with which a contract is being called.
pub struct CallData {
    /// The decoded message selector.
    selector: MessageHandlerSelector,
    /// The raw undecoded parameter bytes.
    raw_params: Vec<u8>,
}

impl Decode for CallData {
    fn decode<I: scale::Input>(input: &mut I) -> CoreResult<Self, scale::Error> {
        let selector = MessageHandlerSelector::decode(input)?;
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
        <Msg as Message>::Input: scale::Encode,
    {
        use scale::Encode;
        Self {
            selector: <Msg as Message>::ID,
            raw_params: args.encode(),
        }
    }
}

/// A hash to identify a called function.
#[derive(Copy, Clone, PartialEq, Eq, Decode)]
pub struct MessageHandlerSelector([u8; 4]);

impl MessageHandlerSelector {
    /// Creates a new message handler selector from the given value.
    pub const fn new(raw: [u8; 4]) -> Self {
        Self(raw)
    }
}

/// A read-only message handler.
///
/// Read-only message handlers cannot mutate contract state.
pub struct MessageHandler<Msg, State, Env>
where
    Msg: Message,
    State: ContractState,
    Env: env::Env,
{
    /// Required in order to trick Rust into thinking that it actually owns a message.
    ///
    /// However, in general message types are zero-sized-types (ZST).
    msg_marker: PhantomData<Msg>,
    /// The actual mutable handler for the message and state.
    raw_handler: RawMessageHandler<Msg, State, Env>,
}

impl<Msg, State, Env> MessageHandler<Msg, State, Env>
where
    Msg: Message,
    State: ContractState,
    Env: env::Env,
{
    /// Returns the associated handler selector.
    pub const fn selector() -> MessageHandlerSelector {
        <Msg as Message>::ID
    }
}

impl<Msg, State, Env> Copy for MessageHandler<Msg, State, Env>
where
    Msg: Message,
    State: ContractState,
    Env: env::Env,
{
}

impl<Msg, State, Env> Clone for MessageHandler<Msg, State, Env>
where
    Msg: Message,
    State: ContractState,
    Env: env::Env,
{
    fn clone(&self) -> Self {
        Self {
            msg_marker: self.msg_marker,
            raw_handler: self.raw_handler,
        }
    }
}

impl<Msg, State, Env> MessageHandler<Msg, State, Env>
where
    Msg: Message,
    State: ContractState,
    Env: env::Env,
{
    /// Constructs a message handler from its raw counterpart.
    pub const fn from_raw(raw_handler: RawMessageHandler<Msg, State, Env>) -> Self {
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
pub struct MessageHandlerMut<Msg, State, Env>
where
    Msg: Message,
    State: ContractState,
    Env: env::Env,
{
    /// Required in order to trick Rust into thinking that it actually owns a message.
    ///
    /// However, in general message types are zero-sized-types (ZST).
    msg_marker: PhantomData<Msg>,
    /// The actual read-only handler for the message and state.
    raw_handler: RawMessageHandlerMut<Msg, State, Env>,
}

impl<Msg, State, Env> Copy for MessageHandlerMut<Msg, State, Env>
where
    Msg: Message,
    State: ContractState,
    Env: env::Env,
{
}

impl<Msg, State, Env> Clone for MessageHandlerMut<Msg, State, Env>
where
    Msg: Message,
    State: ContractState,
    Env: env::Env,
{
    fn clone(&self) -> Self {
        Self {
            msg_marker: self.msg_marker,
            raw_handler: self.raw_handler,
        }
    }
}

impl<Msg, State, Env> MessageHandlerMut<Msg, State, Env>
where
    Msg: Message,
    State: ContractState,
    Env: env::Env,
{
    /// Constructs a message handler from its raw counterpart.
    pub const fn from_raw(raw_handler: RawMessageHandlerMut<Msg, State, Env>) -> Self {
        Self {
            msg_marker: PhantomData,
            raw_handler,
        }
    }
}

impl<Msg, State, Env> MessageHandlerMut<Msg, State, Env>
where
    Msg: Message,
    State: ContractState,
    Env: env::Env,
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
pub trait HandleCall<State, Env> {
    /// Handles the call and returns the encoded result.
    fn handle_call(
        &self,
        env: &mut ExecutionEnv<State, Env>,
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

impl<State, Env> HandleCall<State, Env> for UnreachableMessageHandler {
    fn handle_call(
        &self,
        _env: &mut ExecutionEnv<State, Env>,
        _data: CallData,
    ) -> Result<Vec<u8>> {
        Err(Error::InvalidFunctionSelector)
    }
}

macro_rules! impl_handle_call_for_chain {
    ( $msg_handler_kind:ident, requires_flushing: $requires_flushing:literal ) => {
        impl<Msg, State, Env> HandleCall<State, Env>
            for $msg_handler_kind<Msg, State, Env>
        where
            Msg: Message,
            <Msg as Message>::Output: scale::Encode,
            State: ContractState,
            Env: env::Env,
        {
            fn handle_call(
                &self,
                env: &mut ExecutionEnv<State, Env>,
                data: CallData,
            ) -> Result<Vec<u8>> {
                let args = <Msg as Message>::Input::decode(&mut &data.params()[..])
                    .map_err(|_| Error::InvalidArguments)?;
                let result = (self.raw_handler)(env, args);
                if $requires_flushing {
                    env.state.flush()
                }
                use scale::Encode;
                Ok(result.encode())
            }
        }

        impl<Msg, State, Env, Rest> HandleCall<State, Env>
            for ($msg_handler_kind<Msg, State, Env>, Rest)
        where
            Msg: Message,
            <Msg as Message>::Output: 'static,
            State: ContractState,
            Env: env::Env,
            Rest: HandleCall<State, Env>,
        {
            fn handle_call(
                &self,
                env: &mut ExecutionEnv<State, Env>,
                data: CallData,
            ) -> Result<Vec<u8>> {
                let (handler, rest) = self;
                if $msg_handler_kind::<Msg, State, Env>::selector() == data.selector() {
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
