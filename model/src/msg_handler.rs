use crate::{
	msg::{
		Message,
	},
	exec_env::{
		ExecutionEnv,
	},
	state::{
		ContractState,
	},
};
use pdsl_core::memory::vec::Vec;
use core::{
	marker::PhantomData,
	result::Result as CoreResult,
};
use parity_codec::Decode;
use either::Either;

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

impl parity_codec::Decode for CallData {
	fn decode<I: parity_codec::Input>(input: &mut I) -> Option<Self> {
		let selector = MessageHandlerSelector::decode(input)?;
		let mut param_buf = Vec::new();
		while let Some(byte) = input.read_byte() {
			param_buf.push(byte)
		}
		Some(Self{
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

		// Should produce a hash out of a byte sequence
		// that contains signatures of the following parts:
		//
		// - State::NAME
		// - Msg::NAME
		// - Msg::Input
		// - Msg::Output
		//
		// # Structure
		//
		// State::NAME
		// ~ b'0xFF' ~ Msg::NAME
		// ~ $( b'0xFE' ~ Msg::Input::type_byte_seq() ~ b'0xFD' )*
		// ~ b'0xFD' ~ Msg::Output::type_byte_seq()
		//
		// Where ~ is the byte concat operator.
		// Note that State::NAME, Msg::NAME and everything returned
		// from T::type_byte_seq must be valid ascii so the guard
		// patterns (b'0xFF', b'0xFE', b'0xFD') are unique.
		//
		// Afterwards we hash this sequence by a const hasher
		// to retrieve the resulting MessageHandlerSelector.
		//
		// # Example
		//
		// With State being
		//
		// struct Adder { ... }
		//
		// and Msg being
		//
		// Inc(by: u32) -> u32;
		//
		// We have the following byte sequence:
		//
		// Adder 0xFF Inc 0xFE i32::type_byte_seq() 0xFD i32::type_byte_seq()
	}
}

impl<Msg, State> Copy for MessageHandler<Msg, State>
where
	Msg: Message,
	State: ContractState,
{}

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
		Self { msg_marker: PhantomData, raw_handler }
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
	raw_handler: RawMessageHandlerMut<Msg, State>
}

impl<Msg, State> Copy for MessageHandlerMut<Msg, State>
where
	Msg: Message,
	State: ContractState,
{}

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
		Self { msg_marker: PhantomData, raw_handler }
	}
}

impl<Msg, State> MessageHandlerMut<Msg, State>
where
	Msg: Message,
	State: ContractState,
{
	/// Returns the associated handler selector.
	pub const fn selector() -> MessageHandlerSelector {
		MessageHandlerSelector(0x0) // TODO: Specify and implement behaviour.
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

/// Results of message handling operations.
pub type Result<T> = CoreResult<T, Error>;

/// Types implementing this trait can handle contract calls.
pub trait HandleCall<State> {
	/// The return type of the handled message.
    type Output: /*Response + */ 'static;

	/// Handles the call and returns the result.
	fn handle_call(&self, env: &mut ExecutionEnv<State>, data: CallData) -> Result<Self::Output>;
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
	type Output = ();

	fn handle_call(&self, _env: &mut ExecutionEnv<State>, data: CallData) -> Result<Self::Output> {
		Err(Error::InvalidFunctionSelector)
	}
}

macro_rules! impl_handle_call_for_chain {
	( $msg_handler_kind:ident, requires_flushing: $requires_flushing:literal ) => {
		impl<Msg, State> HandleCall<State> for $msg_handler_kind<Msg, State>
		where
			Msg: Message,
			<Msg as Message>::Output: 'static, // TODO: Could be less restricted.
			State: ContractState,
		{
			type Output = <Msg as Message>::Output;

			fn handle_call(&self, env: &mut ExecutionEnv<State>, data: CallData) -> Result<Self::Output> {
				let args = <Msg as Message>::Input::decode(&mut &data.params()[..])
					.ok_or(Error::InvalidArguments)?;
				use core::intrinsics::type_id;
				let result = (self.raw_handler)(env, args);
				if unsafe { type_id::<<Msg as Message>::Output>() != type_id::<()>() } {
					// Since specialization is not yet implemented in Rust
					// we have to do a manual static dispatch and only return
					// if the messages return type if not equal to `()`.
					if $requires_flushing {
						env.state.flush();
					}
					Ok(result)
				} else {
					// If there was an actual result we want to return it now.
					// Note that `env.return` will end contract execution.
					if $requires_flushing {
						env.state.flush();
					}
					env.r#return(result)
				}
			}
		}

		impl<Msg, State, Rest> HandleCall<State> for ($msg_handler_kind<Msg, State>, Rest)
		where
			Msg: Message,
			<Msg as Message>::Output: 'static,
			State: ContractState,
			Rest: HandleCall<State>,
		{
			type Output = 
				Either<
					<Msg as Message>::Output,
					<Rest as HandleCall<State>>::Output
				>;

			fn handle_call(&self, env: &mut ExecutionEnv<State>, data: CallData) -> Result<Self::Output> {
				let (handler, rest) = self;
				if $msg_handler_kind::<Msg, State>::selector() == data.selector() {
					handler.handle_call(env, data).map(Either::Left)
				} else {
					rest.handle_call(env, data).map(Either::Right)
				}
			}
		}
	}
}

impl_handle_call_for_chain!(MessageHandler, requires_flushing: false);
impl_handle_call_for_chain!(MessageHandlerMut, requires_flushing: true);
