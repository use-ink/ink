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

/// A message with an expected input type and output (result) type.
pub trait Message {
    /// The expected input type, also known as parameter types.
    type Input: scale::Decode;

    /// The output of the message, also known as return type.
    type Output: scale::Encode + 'static;

    /// The user provided message selector.
    ///
    /// This identifier must be unique for every message.
    const ID: MessageHandlerSelector;

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
    // There are three macros to handle the different cases of
    // `$prefix => Foo(maybe_args, …) -> maybe_return_type;`
    //  where `$prefix` can be either `[a, b, c, d]` or `[a; 4]`.

    // Matches `[a, b, c, d] => Foo(maybe_args, …) -> return type;`.
    (
        $( #[$msg_meta:meta] )*
        [ $a:literal, $b:literal, $c:literal, $d:literal $(,)? ] => $msg_name:ident (
            $( $param_name:ident : $param_ty:ty ),*
        ) -> $ret_ty:ty ;
        $($rest:tt)*
    ) => {
        $( #[$msg_meta] )*
        #[derive(Copy, Clone)]
        pub(crate) struct $msg_name;

        impl $crate::Message for $msg_name {
            type Input = ($($param_ty),*);
            type Output = $ret_ty;

            const ID: $crate::MessageHandlerSelector =
                $crate::MessageHandlerSelector::new([$a, $b, $c, $d]);
            const NAME: &'static str = stringify!($msg_name);
        }

        messages!($($rest)*);
    };

    // Matches `[a, b, c, d] => Foo(maybe_args, …);` (no return type).
    (
        $( #[$msg_meta:meta] )*
        [ $a:literal, $b:literal, $c:literal, $d:literal $(,)? ] => $msg_name:ident (
            $( $param_name:ident : $param_ty:ty ),*
        ) ;
        $($rest:tt)*
    ) => {
        messages!(
            $( #[$msg_meta] )*
            [$a, $b, $c, $d] => $msg_name (
                $( $param_name : $param_ty ),*
            ) -> ();
            $($rest)*
        );
    };

    // Matches
    // * `[a; 4] => Foo(maybe_args, …) -> return type;` and
    // * `[a; 4] => Foo(maybe_args, …);`
    (
        $( #[$msg_meta:meta] )*
        [ $a:literal; 4 $(,)? ] => $msg_name:ident (
            $( $param_name:ident : $param_ty:ty ),*
        ) $(-> $maybe_ret:tt)*;
        $($rest:tt)*
    ) => {
        messages!(
            $( #[$msg_meta] )*
            [$a, $a, $a, $a] => $msg_name (
                $( $param_name : $param_ty ),*
            ) $(-> $maybe_ret)* ;
            $($rest)*
        );
    };

    () => {};
}
