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
