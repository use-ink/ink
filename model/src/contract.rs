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
    msg_handler::{
        CallData,
        MessageHandler,
        MessageHandlerMut,
        RawMessageHandler,
        RawMessageHandlerMut,
        UnreachableMessageHandler,
    },
    state::ContractState,
};
use core::marker::PhantomData;
use ink_core::{
    env,
    memory::vec::Vec,
};

/// A marker struct to tell that the deploy handler requires no arguments.
#[derive(Copy, Clone)]
pub struct NoDeployArgs;

/// A handler specific to deploying a smart contract.
///
/// # Note
///
/// This is normally mainly used to correctly initialize
/// a smart contracts state.
pub struct DeployHandler<State, Env, Args> {
    /// The arguments that deploy expects.
    ///
    /// This tricks Rust into thinking that this owns the state type.
    /// However, it is just a marker which allows the contract declaration
    /// to be a zero-sized-type (ZST).
    args: PhantomData<Args>,
    /// The actual deployment function.
    deploy_fn: fn(&mut ExecutionEnv<State, Env>, Args),
}

impl<State, Env> DeployHandler<State, Env, NoDeployArgs> {
    /// Returns a deploy handler that does nothing.
    const fn init() -> Self {
        Self {
            args: PhantomData,
            deploy_fn: move |_env, _| {},
        }
    }
}

impl<State, Env, Args> DeployHandler<State, Env, Args> {
    /// Returns a new deploy handler for the given closure.
    const fn new(raw_handler: fn(&mut ExecutionEnv<State, Env>, Args)) -> Self {
        Self {
            args: PhantomData,
            deploy_fn: raw_handler,
        }
    }
}

impl<State, Env, Args> Copy for DeployHandler<State, Env, Args> {}

impl<State, Env, Args> Clone for DeployHandler<State, Env, Args> {
    fn clone(&self) -> Self {
        Self {
            args: self.args,
            deploy_fn: self.deploy_fn,
        }
    }
}

/// A contract declaration.
///
/// Uses the builder pattern in order to represent a contract
/// based on compile-time construction.
///
/// Can be used to actually instantiate a contract during run-time
/// in order to dispatch a contract call or deploy state.
pub struct ContractDecl<State, Env, DeployArgs, HandlerChain> {
    /// The type of the contract state.
    ///
    /// This tricks Rust into thinking that this owns the state type.
    /// However, it is just a marker which allows the contract declaration
    /// to be a zero-sized-type (ZST).
    state: PhantomData<State>,

    deployer: DeployHandler<State, Env, DeployArgs>,
    /// The compile-time chain of message handlers.
    ///
    /// # Note
    ///
    /// They are represented by a recursive tuple chain and start with
    /// a simple `UnreachableMessageHandler` node. For every further
    /// registered message handler this tuple is extended recursively.
    ///
    /// ## Example
    ///
    /// ```no_compile
    /// UnreachableMessageHandler               // Upon initialization.
    /// (Foo, UnreachableMessageHandler)        // After adding message handler Foo.
    /// (Bar, (Foo, UnreachableMessageHandler)) // After adding message handler Bar.
    /// ```
    ///
    /// Note that every pair of message handlers is also a message handler.
    handlers: HandlerChain,
}

impl<State, Env, DeployArgs, HandlerChain> Clone
    for ContractDecl<State, Env, DeployArgs, HandlerChain>
where
    HandlerChain: Clone,
{
    fn clone(&self) -> Self {
        Self {
            state: self.state,
            deployer: self.deployer,
            handlers: self.handlers.clone(),
        }
    }
}

impl<State, Env, DeployArgs, HandlerChain> Copy
    for ContractDecl<State, Env, DeployArgs, HandlerChain>
where
    HandlerChain: Copy,
{
}

/// An empty contract state.
#[derive(Copy, Clone)]
pub struct EmptyContractState;

/// An empty env.
#[derive(Copy, Clone)]
pub struct EmptyEnv;

impl ContractDecl<EmptyContractState, EmptyEnv, NoDeployArgs, UnreachableMessageHandler> {
    /// Creates a new contract declaration with the given name.
    pub const fn using<State, Env>(
    ) -> ContractDecl<State, Env, NoDeployArgs, UnreachableMessageHandler> {
        ContractDecl {
            state: PhantomData,
            deployer: DeployHandler::init(),
            handlers: UnreachableMessageHandler,
        }
    }
}

impl<State, Env> ContractDecl<State, Env, NoDeployArgs, UnreachableMessageHandler> {
    /// Registers the given deployment procedure for the contract.
    ///
    /// # Note
    ///
    /// This is used to initialize the contract state upon deployment.
    pub const fn on_deploy<Args>(
        self,
        handler: fn(&mut ExecutionEnv<State, Env>, Args),
    ) -> ContractDecl<State, Env, Args, UnreachableMessageHandler>
    where
        Args: scale::Decode,
    {
        ContractDecl {
            state: self.state,
            deployer: DeployHandler::new(handler),
            handlers: self.handlers,
        }
    }
}

impl<State, Env, DeployArgs, HandlerChain>
    ContractDecl<State, Env, DeployArgs, HandlerChain>
where
    Self: Copy, // Required in order to make this compile-time computable.
{
    /// Convenience method to append another message handler.
    const fn append_msg_handler<MsgHandler>(
        self,
        handler: MsgHandler,
    ) -> ContractDecl<State, Env, DeployArgs, (MsgHandler, HandlerChain)> {
        ContractDecl {
            state: PhantomData,
            deployer: self.deployer,
            handlers: (handler, self.handlers),
        }
    }

    /// Registers a read-only message handler.
    ///
    /// # Note
    ///
    /// Read-only message handlers do not mutate contract state.
    pub const fn on_msg<Msg>(
        self,
        handler: RawMessageHandler<Msg, State, Env>,
    ) -> ContractDecl<
        State,
        Env,
        DeployArgs,
        (MessageHandler<Msg, State, Env>, HandlerChain),
    >
    where
        Msg: Message,
        State: ContractState,
        Env: env::Env,
    {
        self.append_msg_handler(MessageHandler::from_raw(handler))
    }

    /// Registers a mutable message handler.
    ///
    /// # Note
    ///
    /// Mutable message handlers may mutate contract state.
    pub const fn on_msg_mut<Msg>(
        self,
        handler: RawMessageHandlerMut<Msg, State, Env>,
    ) -> ContractDecl<
        State,
        Env,
        DeployArgs,
        (MessageHandlerMut<Msg, State, Env>, HandlerChain),
    >
    where
        Msg: Message,
        State: ContractState,
        Env: env::Env,
    {
        self.append_msg_handler(MessageHandlerMut::from_raw(handler))
    }
}

impl<State, Env, DeployArgs, HandlerChain>
    ContractDecl<State, Env, DeployArgs, HandlerChain>
where
    // Self: Copy, // Required in order to make this compile-time computable.
    State: ContractState,
{
    /// Creates an instance of the contract declaration.
    ///
    /// This associates the state with the contract storage
    /// and defines its layout.
    pub fn instantiate(self) -> ContractInstance<State, Env, DeployArgs, HandlerChain> {
        use ink_core::storage::{
            alloc::{
                AllocateUsing,
                BumpAlloc,
            },
            Key,
        };
        let env = unsafe {
            // Note that it is totally fine here to start with a key
            // offset of `0x0` as long as we only consider having one
            // contract instance per execution. Otherwise their
            // associated storage could overlap.
            //
            // This can later be solved by having an implementation for
            // `AllocateUsing` for `ContractDecl` to actually instantiate
            // them using an already existing allocator. Note that then
            // all contracts always have to be allocated in the same
            // order which could be achieved by simply putting all contracts
            // into a contract struct that itself implements `AllocateUsing`.
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            AllocateUsing::allocate_using(&mut alloc)
        };
        ContractInstance {
            env,
            deployer: self.deployer,
            handlers: self.handlers,
        }
    }
}

/// A return status code for `deploy` and `dispatch` calls back to the SRML contracts module.
///
/// # Note
///
/// The `call` and `create` SRML contracts interfacing
/// instructions both return a `u32`, however, only the least-significant
/// 8 bits can be non-zero.
/// For a start we only allow `0` and `255` as return codes.
///
/// Zero (`0`) represents a successful execution, (`255`) means invalid
/// execution (e.g. trap) and any value in between represents a non-
/// specified invalid execution.
///
/// Other error codes are subject to future proposals.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RetCode(u8);

impl RetCode {
    /// Indicates a successful execution.
    pub fn success() -> Self {
        Self(0)
    }

    /// Indicates a failure upon execution.
    pub fn failure() -> Self {
        Self(255)
    }

    /// Returns the internal `u32` value.
    pub fn to_u32(self) -> u32 {
        self.0 as u32
    }
}

/// A simple interface to work with contracts.
pub trait Contract {
    /// Deploys the contract state.
    ///
    /// Should be performed exactly once during contract lifetime.
    /// Consumes the contract since nothing should be done afterwards.
    fn deploy(self) -> RetCode;

    /// Dispatches the call input to a pre defined
    /// contract message and runs its handler.
    ///
    /// Consumes self since it should be the default
    /// action after instantiation.
    ///
    /// # Panics
    ///
    /// Panics (Wasm: traps) if the call input was invalid.
    /// The call input is invalid if there was no matching
    /// function selector found or if the data for a given
    /// selected function was not decodable.
    fn dispatch(self) -> RetCode;
}

/// An interface that allows for simple testing of contracts.
pub trait TestableContract {
    /// The arguments used for deployment.
    ///
    /// These must be the same as the ones defined on the deploy handler
    /// of a contract declaration.
    type DeployArgs: scale::Encode;

    /// Deploys the contract given the provided arguments for deployment.
    ///
    /// # Note
    ///
    /// This shall be performed only once during the lifetime of a contract.
    ///
    /// # Panics
    ///
    /// This might panic if the provided arguments do not match the expected.
    fn deploy(&mut self, deploy_args: Self::DeployArgs);

    /// Calls the contract with the given message and its
    /// inputs and upon successful execution returns its result.
    ///
    /// # Note
    ///
    /// Takes `&mut self` since it could potentially call a message
    /// that mutates state. There currently is no separation between
    /// messages that mutate state and those that do not.
    ///
    /// # Panics
    ///
    /// If the contract has no message handler setup for the given message.
    fn call<Msg>(&mut self, input: <Msg as Message>::Input) -> <Msg as Message>::Output
    where
        Msg: Message,
        <Msg as Message>::Input: scale::Encode,
        <Msg as Message>::Output: scale::Decode;
}

/// An instance of a contract.
///
/// This resembles the concrete contract that is the result of
/// an instantiation of a contract declaration.
pub struct ContractInstance<State, Env, DeployArgs, HandlerChain> {
    /// The execution environment that is wrapping the actual state.
    env: ExecutionEnv<State, Env>,
    /// The deploy functionality.
    deployer: DeployHandler<State, Env, DeployArgs>,
    /// The contract's message handlers.
    handlers: HandlerChain,
}

impl<State, Env, DeployArgs, HandlerChain> Contract
    for ContractInstance<State, Env, DeployArgs, HandlerChain>
where
    State: ContractState,
    Env: env::Env,
    DeployArgs: scale::Decode,
    HandlerChain: crate::HandleCall<State, Env>,
{
    /// Deploys the contract.
    ///
    /// This runs exactly once during the lifetime of a contract and
    /// is used to initialize the contract's state.
    ///
    /// # Note
    ///
    /// Accessing uninitialized contract state can end in trapping execution
    /// or in the worst case in undefined behaviour.
    fn deploy(self) -> RetCode {
        // Deploys the contract state.
        //
        // Should be performed exactly once during contract lifetime.
        // Consumes the contract since nothing should be done afterwards.
        let input = Env::input();
        let mut this = self;
        if let Err(err) = this.deploy_with(input.as_slice()) {
            return err
        }
        core::mem::forget(this.env);
        RetCode::success()
    }

    /// Dispatches the input buffer and calls the associated message.
    ///
    /// Returns the result to the caller if there is any.
    fn dispatch(self) -> RetCode {
        // Dispatches the given input to a pre defined
        // contract message and runs its handler.
        //
        // Consumes self since it should be the default
        // action after instantiation.
        //
        // Internally calls the associated call<Msg>.
        use scale::Decode;
        let input = Env::input();
        let call_data = CallData::decode(&mut &input[..]).unwrap();
        let mut this = self;
        if let Err(err) = this.call_with_and_return(call_data) {
            return err
        }
        core::mem::forget(this.env);
        RetCode::success()
    }
}

impl<State, Env, DeployArgs, HandlerChain>
    ContractInstance<State, Env, DeployArgs, HandlerChain>
where
    State: ContractState,
    Env: env::Env,
    DeployArgs: scale::Decode,
    HandlerChain: crate::HandleCall<State, Env>,
{
    /// Deploys the contract.
    ///
    /// This runs exactly once during the lifetime of a contract and
    /// is used to initialize the contract's state.
    ///
    /// # Note
    ///
    /// Accessing uninitialized contract state can end in trapping execution
    /// or in the worst case in undefined behaviour.
    fn deploy_with(&mut self, input: &[u8]) -> Result<(), RetCode> {
        // Deploys the contract state.
        //
        // Should be performed exactly once during contract lifetime.
        // Consumes the contract since nothing should be done afterwards.
        use ink_core::storage::alloc::Initialize as _;
        self.env.initialize(());
        let deploy_params =
            DeployArgs::decode(&mut &input[..]).map_err(|_err| RetCode::failure())?;
        (self.deployer.deploy_fn)(&mut self.env, deploy_params);
        self.env.state.flush();
        Ok(())
    }

    /// Calls the message encoded by the given call data
    /// and returns the resulting value back to the caller.
    fn call_with_and_return(&mut self, call_data: CallData) -> Result<(), RetCode> {
        let result = self.call_with(call_data)?;
        if !result.is_empty() {
            self.env.return_data(result)
        }
        Ok(())
    }

    /// Calls the message encoded by the given call data.
    ///
    /// # Panics
    ///
    /// - If the contract has no message handler setup for the
    ///   message that is encoded by the given call data.
    /// - If the encoded input arguments for the message do not
    ///   match the expected format.
    fn call_with(&mut self, call_data: CallData) -> Result<Vec<u8>, RetCode> {
        match self.handlers.handle_call(&mut self.env, call_data) {
            Ok(encoded_result) => Ok(encoded_result),
            Err(_err) => Err(RetCode::failure()),
        }
    }
}

impl<State, Env, DeployArgs, HandlerChain> TestableContract
    for ContractInstance<State, Env, DeployArgs, HandlerChain>
where
    State: ContractState,
    Env: env::Env,
    DeployArgs: scale::Codec,
    HandlerChain: crate::HandleCall<State, Env>,
{
    type DeployArgs = DeployArgs;

    fn deploy(&mut self, input: Self::DeployArgs) {
        self.deploy_with(&input.encode()[..])
            .expect("`deploy` failed to execute properly")
    }

    fn call<Msg>(&mut self, input: <Msg as Message>::Input) -> <Msg as Message>::Output
    where
        Msg: Message,
        <Msg as Message>::Input: scale::Encode,
        <Msg as Message>::Output: scale::Decode,
    {
        let encoded_result = self
            .call_with(CallData::from_msg::<Msg>(input))
            .expect("`call` failed to execute properly");
        use scale::Decode;
        <Msg as Message>::Output::decode(&mut &encoded_result[..])
            .expect("`call_with` only encodes the correct types")
    }
}
