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

use crate::msg_handler::MessageHandlerSelector;

/// Types implementing this trait are usable as contract messages.
pub trait Message {
    /// The expected input type, also known as parameter types.
    type Input: scale::Decode;
    /// The output of the message, also known as return type.
    type Output: scale::Encode + 'static;
    /// The user provided message selector.
    ///
    /// This identifier must be unique for every message.
    const SELECTOR: MessageHandlerSelector;
    /// Whether the message is allowed to mutate state.
    const IS_MUT: bool;
    /// The name of the message.
    ///
    /// # Note
    ///
    /// This must be a valid Rust identifier.
    const NAME: &'static str;
}

/// Defines messages for contracts with less boilerplate code.
#[macro_export]
macro_rules! messages {
    // Rule for `&self` message with a return type.
	(
		$( #[$msg_meta:meta] )*
		$msg_id:literal => $msg_name:ident (
			&self $( , $param_name:ident : $param_ty:ty )* $(,)?
		) -> $ret_ty:ty ;

		$($rest:tt)*
	) => {
		$( #[$msg_meta] )*
		#[derive(Copy, Clone)]
		pub(crate) struct $msg_name;

		impl $crate::Message for $msg_name {
			type Input = ($($param_ty),*);
			type Output = $ret_ty;

            const IS_MUT: bool = false;
			const SELECTOR: $crate::MessageHandlerSelector = $crate::MessageHandlerSelector::new($msg_id);
			const NAME: &'static str = stringify!($msg_name);
		}

        impl $crate::checks::CheckIsMessageMut for $msg_name {
            type Value = [Self; <Self as $crate::Message>::IS_MUT as usize];
        }

		messages!($($rest)*);
	};
    // Rule for `&self` message without a return type.
	(
		$( #[$msg_meta:meta] )*
		$msg_id:literal => $msg_name:ident (
			&self $( , $param_name:ident : $param_ty:ty )* $(,)?
		) ;

		$($rest:tt)*
	) => {
		messages!(
			$( #[$msg_meta] )*
			$msg_id => $msg_name (
				&self $( , $param_name : $param_ty )*
			) -> ();

			$($rest)*
		);
	};
    // Rule for `&mut self` message with a return type.
	(
		$( #[$msg_meta:meta] )*
		$msg_id:literal => $msg_name:ident (
			&mut self $( , $param_name:ident : $param_ty:ty )* $(,)?
		) -> $ret_ty:ty ;

		$($rest:tt)*
	) => {
		$( #[$msg_meta] )*
		#[derive(Copy, Clone)]
		pub(crate) struct $msg_name;

		impl $crate::Message for $msg_name {
			type Input = ($($param_ty),*);
			type Output = $ret_ty;

            const IS_MUT: bool = true;
			const SELECTOR: $crate::MessageHandlerSelector = $crate::MessageHandlerSelector::new($msg_id);
			const NAME: &'static str = stringify!($msg_name);
		}

        impl $crate::checks::CheckIsMessageMut for $msg_name {
            type Value = [Self; <Self as $crate::Message>::IS_MUT as usize];
        }

		messages!($($rest)*);
	};
    // Rule for `&mut self` message without a return type.
	(
		$( #[$msg_meta:meta] )*
		$msg_id:literal => $msg_name:ident (
			&mut self $( , $param_name:ident : $param_ty:ty )* $(,)?
		) ;

		$($rest:tt)*
	) => {
		messages!(
			$( #[$msg_meta] )*
			$msg_id => $msg_name (
				&mut self $( , $param_name : $param_ty )*
			) -> ();

			$($rest)*
		);
	};
    // Base rule to end macro.
	() => {};
}
