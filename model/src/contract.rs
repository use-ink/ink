// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
	state::{
		ContractState,
	},
	exec_env::{
		ExecutionEnv,
	},
	msg::{
		Message,
	},
	msg_handler::{
		CallData,
		UnreachableMessageHandler,
		MessageHandler,
		MessageHandlerMut,
		RawMessageHandler,
		RawMessageHandlerMut,
	},
};
use core::marker::PhantomData;

/// A marker struct to tell that the deploy handler requires no arguments.
#[derive(Copy, Clone)]
pub struct NoDeployArgs;

/// A handler specific to deploying a smart contract.
///
/// # Note
///
/// This is normally mainly used to correctly initialize
/// a smart contracts state.
pub struct DeployHandler<State, Args> {
	/// The arguments that deploy expects.
	///
	/// This tricks Rust into thinking that this owns the state type.
	/// However, it is just a marker which allows the contract declaration
	/// to be a zero-sized-type (ZST).
	args: PhantomData<Args>,
	/// The actual deployment function.
	deploy_fn: fn(&mut ExecutionEnv<State>, Args),
}

impl<State> DeployHandler<State, NoDeployArgs> {
	/// Returns a deploy handler that does nothing.
	const fn init() -> Self {
		Self {
			args: PhantomData,
			deploy_fn: move |_env, _| {},
		}
	}
}

impl<State, Args> DeployHandler<State, Args> {
	/// Returns a new deploy handler for the given closure.
	const fn new(raw_handler: fn(&mut ExecutionEnv<State>, Args)) -> Self {
		Self {
			args: PhantomData,
			deploy_fn: raw_handler,
		}
	}
}

impl<State, Args> Copy for DeployHandler<State, Args> {}

impl<State, Args> Clone for DeployHandler<State, Args> {
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
pub struct ContractDecl<
	State,
	DeployArgs,
	HandlerChain,
> {
	/// The type of the contract state.
	///
	/// This tricks Rust into thinking that this owns the state type.
	/// However, it is just a marker which allows the contract declaration
	/// to be a zero-sized-type (ZST).
	state: PhantomData<State>,

	deployer: DeployHandler<State, DeployArgs>,
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

impl<State, DeployArgs, HandlerChain> Clone for ContractDecl<State, DeployArgs, HandlerChain>
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

impl<State, DeployArgs, HandlerChain> Copy for ContractDecl<State, DeployArgs, HandlerChain>
where
	HandlerChain: Copy,
{}

/// An empty contract state.
#[derive(Copy, Clone)]
pub struct EmptyContractState;

impl ContractDecl<EmptyContractState, NoDeployArgs, UnreachableMessageHandler> {
	/// Creates a new contract declaration with the given name.
	pub const fn using<State>() -> ContractDecl<State, NoDeployArgs, UnreachableMessageHandler> {
		ContractDecl {
			state: PhantomData,
			deployer: DeployHandler::init(),
			handlers: UnreachableMessageHandler,
		}
	}
}

impl<State> ContractDecl<State, NoDeployArgs, UnreachableMessageHandler> {
	/// Registers the given deployment procedure for the contract.
	///
	/// # Note
	///
	/// This is used to initialize the contract state upon deployment.
	pub const fn on_deploy<Args>(
		self,
		handler: fn(&mut ExecutionEnv<State>, Args),
	)
		-> ContractDecl<State, Args, UnreachableMessageHandler>
	where
		Args: parity_codec::Decode,
	{
		ContractDecl {
			state: self.state,
			deployer: DeployHandler::new(handler),
			handlers: self.handlers,
		}
	}
}

impl<State, DeployArgs, HandlerChain> ContractDecl<State, DeployArgs, HandlerChain>
where
	Self: Copy, // Required in order to make this compile-time computable.
{
	/// Convenience method to append another message handler.
	const fn append_msg_handler<MsgHandler>(self, handler: MsgHandler)
		-> ContractDecl<State, DeployArgs, (MsgHandler, HandlerChain)>
	{
		ContractDecl {
			state: PhantomData,
			deployer: self.deployer,
			handlers: (handler, self.handlers)
		}
	}

	/// Registers a read-only message handler.
	///
	/// # Note
	///
	/// Read-only message handlers do not mutate contract state.
	pub const fn on_msg<Msg>(
		self,
		handler: RawMessageHandler<Msg, State>,
	)
		-> ContractDecl<State, DeployArgs, (MessageHandler<Msg, State>, HandlerChain)>
	where
		Msg: Message,
		State: ContractState,
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
		handler: RawMessageHandlerMut<Msg, State>,
	)
		-> ContractDecl<State, DeployArgs, (MessageHandlerMut<Msg, State>, HandlerChain)>
	where
		Msg: Message,
		State: ContractState,
	{
		self.append_msg_handler(MessageHandlerMut::from_raw(handler))
	}
}

impl<State, DeployArgs, HandlerChain> ContractDecl<State, DeployArgs, HandlerChain>
where
	// Self: Copy, // Required in order to make this compile-time computable.
	State: ContractState,
{
	/// Creates an instance of the contract declaration.
	///
	/// This assocates the state with the contract storage
	/// and defines its layout.
	pub fn instantiate(self) -> ContractInstance<State, DeployArgs, HandlerChain> {
		use pdsl_core::{
			storage::{
				Key,
				alloc::{
					BumpAlloc,
					AllocateUsing,
				},
			},
		};
		let state: State = unsafe {
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
			env: ExecutionEnv::new(state),
			deployer: self.deployer,
			handlers: self.handlers,
		}
	}
}

/// A simple interface to work with contracts.
pub trait Contract {
	/// Deploys the contract state.
	///
	/// Should be performed exactly once during contract lifetime.
	/// Consumes the contract since nothing should be done afterwards.
	fn deploy(self);

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
	fn dispatch(self);
}

/// An interface that allows for simple testing of contracts.
pub trait TestableContract: Contract {
	/// Calls the contract with the given message and its
	/// inputs and upon successful execution returns its result.
	fn call<Msg>(&mut self, input: <Msg as Message>::Input) -> <Msg as Message>::Output
	where
		Msg: Message,
		<Msg as Message>::Input: parity_codec::Encode,
		<Msg as Message>::Output: parity_codec::Decode;
}

/// An instance of a contract.
///
/// This resembles the concrete contract that is the result of
/// an instantiation of a contract declaration.
pub struct ContractInstance<State, DeployArgs, HandlerChain> {
	/// The execution environment that is wrapping the actual state.
	env: ExecutionEnv<State>,
	/// The deploy functionality.
	deployer: DeployHandler<State, DeployArgs>,
	/// The contract's message handlers.
	handlers: HandlerChain,
}

impl<State, DeployArgs, HandlerChain> Contract for ContractInstance<State, DeployArgs, HandlerChain>
where
	State: ContractState,
	DeployArgs: parity_codec::Decode,
	HandlerChain: crate::HandleCall<State>,
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
	fn deploy(self) {
		// Deploys the contract state.
		//
		// Should be performed exactly once during contract lifetime.
		// Consumes the contract since nothing should be done afterwards.
		use pdsl_core::env::Env;
		let input = pdsl_core::env::ContractEnv::input();
		let deploy_params = DeployArgs::decode(&mut &input[..]).unwrap();
		let mut this = self;
		(this.deployer.deploy_fn)(&mut this.env, deploy_params);
		this.env.state.flush()
	}

	/// Dispatches the input buffer and calls the associated message.
	///
	/// Returns the result to the caller if there is any.
	fn dispatch(self) {
		// Dispatches the given input to a pre defined
		// contract message and runs its handler.
		//
		// Consumes self since it should be the default
		// action after instantiation.
		//
		// Internally calls the associated call<Msg>.
		use pdsl_core::env::Env;
		use parity_codec::Decode;
		let input = pdsl_core::env::ContractEnv::input();
		let call_data = CallData::decode(&mut &input[..]).unwrap();
		let mut this = self;
		this.call_with_and_return(call_data)
	}
}

impl<State, DeployArgs, HandlerChain> ContractInstance<State, DeployArgs, HandlerChain>
where
	State: ContractState,
	DeployArgs: parity_codec::Decode,
	HandlerChain: crate::HandleCall<State>,
{
	/// Calls the message encoded by the given call data
	/// and returns the resulting value back to the caller.
	fn call_with_and_return(&mut self, call_data: CallData) {
		let encoded_result = self.call_with(call_data);
		if encoded_result.len() > 0 {
			self.env.r#return(encoded_result)
		}
	}

	/// Calls the message encoded by the given call data.
	///
	/// # Panics
	///
	/// - If the contract has no message handler setup for the
	///   message that is encoded by the given call data.
	/// - If the encoded input arguments for the message do not
	///   match the expected format.
	fn call_with(&mut self, call_data: CallData) -> Vec<u8> {
		match self.handlers.handle_call(&mut self.env, call_data) {
			Ok(encoded_result) => encoded_result,
			Err(err) => {
				panic!(err.description())
			}
		}
	}
}

impl<State, DeployArgs, HandlerChain> TestableContract for ContractInstance<State, DeployArgs, HandlerChain>
where
	State: ContractState,
	DeployArgs: parity_codec::Decode,
	HandlerChain: crate::HandleCall<State>,
{
	/// Calls the given message with its expected input arguments.
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
		<Msg as Message>::Input: parity_codec::Encode,
		<Msg as Message>::Output: parity_codec::Decode,
	{
		let encoded_result = self.call_with(CallData::from_msg::<Msg>(input));
		use parity_codec::Decode;
		<Msg as Message>::Output::decode(&mut &encoded_result[..])
			.expect("`call_with` only encodes the correct types")
	}
}
