
/// A message with an expected input type and output (result) type.
pub trait Message {
	/// The expected input type, also known as parameter types.
	type Input: parity_codec::Decode;

	/// The output of the message, also known as return type.
	type Output: parity_codec::Encode;

	/// The name of the message.
	///
	/// # Note
	///
	/// This should be a valid Rust identifier.
	const NAME: &'static str;
}

/// Defines messages for contracts with less boilerplate code.
#[macro_export]
macro_rules! messages {
	(
		$( #[$msg_meta:meta] )* $msg_name:ident (
			$( $param_name:ident : $param_ty:ty ),*
		) -> $ret_ty:ty ; $($rest:tt)*
	) => {
		$( #[$msg_meta] )*
		#[derive(Copy, Clone)]
		struct $msg_name;

		impl crate::msg::Message for $msg_name {
			type Input = ($($param_ty),*);
			type Output = $ret_ty;

			const NAME: &'static str = stringify!($msg_name);
		}

		messages!($($rest)*);
	};
	( $msg_name:ident ( $( $param_name:ident : $param_ty:ty ),* ) ; $($rest:tt)* ) => {
		messages!( $msg_name ( $( $param_name : $param_ty ),* ) -> (); $($rest)* );
	};
	() => {};
}
