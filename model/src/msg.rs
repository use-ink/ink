use crate::{
	msg_handler::MessageHandlerSelector,
};

/// A message with an expected input type and output (result) type.
pub trait Message {
	/// The expected input type, also known as parameter types.
	type Input: parity_codec::Decode;

	/// The output of the message, also known as return type.
	type Output: parity_codec::Encode + 'static;

	/// The user provided message selector.
	///
	/// This identifier must be unique for every message.
	const ID: MessageHandlerSelector;

	/// The name of the message.
	///
	/// # Note
	///
	/// This must be a valid Rust identifier.
	const NAME: &'static [u8];
}

/// Defines messages for contracts with less boilerplate code.
#[macro_export]
macro_rules! messages {
	(
		$( #[$msg_meta:meta] )*
		$msg_id:literal => $msg_name:ident (
			$( $param_name:ident : $param_ty:ty ),*
		) -> $ret_ty:ty ;

		$($rest:tt)*
	) => {
		$( #[$msg_meta] )*
		#[derive(Copy, Clone)]
		struct $msg_name;

		impl $crate::Message for $msg_name {
			type Input = ($($param_ty),*);
			type Output = $ret_ty;

			const ID: $crate::MessageHandlerSelector = $crate::MessageHandlerSelector::new($msg_id);
			const NAME: &'static [u8] = stringify!($msg_name).as_bytes();
		}

		messages!($($rest)*);
	};
	(
		$( #[$msg_meta:meta] )*
		$msg_id:literal => $msg_name:ident (
			$( $param_name:ident : $param_ty:ty ),*
		) ;

		$($rest:tt)*
	) => {
		messages!(
			$( #[$msg_meta] )*
			$msg_id => $msg_name (
				$( $param_name : $param_ty ),*
			) -> ();

			$($rest)*
		);
	};
	() => {};
}
