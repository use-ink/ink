use crate::{
	state::{
		ContractState,
		EmptyContractState,
	},
	msg::{
		Message,
	},
	msg_handler::{
		UnreachableMessageHandler,
		MessageHandler,
		MessageHandlerMut,
		RawMessageHandler,
		RawMessageHandlerMut,
	},
};
use core::marker::PhantomData;

pub struct ContractDecl<
	State,
	HandlerChain,
> {
	name: &'static str,
	state: PhantomData<State>,
	handlers: HandlerChain,
}

impl<State, HandlerChain> Clone for ContractDecl<State, HandlerChain>
where
	HandlerChain: Clone,
{
	fn clone(&self) -> Self {
		Self {
			name: self.name,
			state: self.state,
			handlers: self.handlers.clone(),
		}
	}
}

impl<State, HandlerChain> Copy for ContractDecl<State, HandlerChain>
where
	HandlerChain: Copy,
{}

impl ContractDecl<EmptyContractState, UnreachableMessageHandler> {
	/// Creates a new contract declaration with the given name.
	pub fn new(name: &'static str) -> Self {
		Self {
			name,
			state: PhantomData,
			handlers: UnreachableMessageHandler,
		}
	}

	/// Makes the contract declaration use the given state.
	pub fn using_state<State>(self) -> ContractDecl<State, UnreachableMessageHandler> {
		ContractDecl {
			name: self.name,
			state: PhantomData,
			handlers: UnreachableMessageHandler,
		}
	}
}

impl<State, HandlerChain> ContractDecl<State, HandlerChain> {
	/// Convenience method to append another message handler.
	const fn append_msg_handler<MsgHandler>(self, handler: MsgHandler)
		-> ContractDecl<State, (MsgHandler, HandlerChain)>
	where
		Self: Copy,
	{
		ContractDecl {
			name: self.name,
			state: PhantomData,
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
		-> ContractDecl<State, (MessageHandler<Msg, State>, HandlerChain)>
	where
		Self: Copy,
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
		-> ContractDecl<State, (MessageHandlerMut<Msg, State>, HandlerChain)>
	where
		Self: Copy,
		Msg: Message,
		State: ContractState,
	{
		self.append_msg_handler(MessageHandlerMut::from_raw(handler))
	}
}
