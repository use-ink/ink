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

use crate::Selector;

/// Constructor and messages implement this to communicate their selectors.
pub trait FnSelector {
    /// The selector.
    const SELECTOR: Selector;
}

/// The input types of either a contructor or message.
pub trait FnInput {
    /// The inputs.
    type Input: scale::Decode;
}

/// The output type of a constructor or message.
pub trait FnOutput {
    /// The output.
    type Output: scale::Encode + 'static;
}

/// The compile-time known name of an entity.
pub trait Named {
    /// The name.
    const NAME: &'static str;
}

/// Types implementing this trait are usable as contract messages.
pub trait Message: FnSelector + FnInput + FnOutput + Named {
    const IS_MUT: bool;
}

/// Types implementing this trait are usable as contract constructors.
pub trait Constructor: Message {}

/// Defines constructors for contracts with less boilerplate code.
#[macro_export]
macro_rules! constructors {
    (
        $( #[$attr:meta] )*
        $selector:literal => $name:ident (
            $( $param_name:ident : $param_ty:ty ),*
        );

        $($rest:tt)*
    ) => {
        $( #[$attr] )*
        #[derive(Copy, Clone)]
        pub(crate) struct $name;

        impl $crate::FnSelector for $name {
            const SELECTOR: $crate::Selector =
                $crate::Selector::new($selector);
        }

        impl $crate::FnInput for $name {
            type Input = ( $($param_ty),* );
        }

        impl $crate::FnOutput for $name {
            type Output = ();
        }

        impl $crate::Named for $name {
            const NAME: &'static str = stringify!($name);
        }

        impl $crate::Message for $name {
            const IS_MUT: bool = true;
        }

        impl $crate::Constructor for $name {}

        constructors!($($rest)*);
    };
    () => {};
}

/// Defines messages for contracts with less boilerplate code.
#[macro_export]
macro_rules! messages {
    // Rule for `&self` message with a return type.
    (
        $( #[$attr:meta] )*
        $selector:literal => $name:ident (
            &self $( , $param_name:ident : $param_ty:ty )* $(,)?
        ) -> $output:ty ;

        $($rest:tt)*
    ) => {
        $( #[$attr] )*
        #[derive(Copy, Clone)]
        pub(crate) enum $name {}

        impl $crate::FnSelector for $name {
            const SELECTOR: $crate::Selector =
                $crate::Selector::new($selector);
        }

        impl $crate::FnInput for $name {
            type Input = ( $($param_ty),* );
        }

        impl $crate::FnOutput for $name {
            type Output = $output;
        }

        impl $crate::Named for $name {
            const NAME: &'static str = stringify!($name);
        }

        impl $crate::Message for $name {
            const IS_MUT: bool = false;
        }

        impl $crate::checks::CheckIsMessageMut for $name {
            type Value = [Self; <Self as $crate::Message>::IS_MUT as usize];
        }

        messages!($($rest)*);
    };
    // Rule for `&self` message without a return type.
    (
        $( #[$attr:meta] )*
        $selector:literal => $name:ident (
            &self $( , $param_name:ident : $param_ty:ty )* $(,)?
        ) ;

        $($rest:tt)*
    ) => {
        messages!(
            $( #[$attr] )*
            $selector => $name (
                &self $( , $param_name : $param_ty )*
            ) -> ();

            $($rest)*
        );
};
    // Rule for `&mut self` message with a return type.
    (
        $( #[$attr:meta] )*
        $selector:literal => $name:ident (
            &mut self $( , $param_name:ident : $param_ty:ty )* $(,)?
        ) -> $output:ty ;

        $($rest:tt)*
    ) => {
        $( #[$attr] )*
        #[derive(Copy, Clone)]
        pub(crate) enum $name {}

        impl $crate::FnSelector for $name {
            const SELECTOR: $crate::Selector =
                $crate::Selector::new($selector);
        }

        impl $crate::FnInput for $name {
            type Input = ( $($param_ty),* );
        }

        impl $crate::FnOutput for $name {
            type Output = $output;
        }

        impl $crate::Named for $name {
            const NAME: &'static str = stringify!($name);
        }

        impl $crate::Message for $name {
            const IS_MUT: bool = true;
        }

        impl $crate::checks::CheckIsMessageMut for $name {
            type Value = [Self; <Self as $crate::Message>::IS_MUT as usize];
        }

        messages!($($rest)*);
    };
    // Rule for `&mut self` message without a return type.
    (
        $( #[$attr:meta] )*
        $selector:literal => $name:ident (
            &mut self $( , $param_name:ident : $param_ty:ty )* $(,)?
        ) ;

        $($rest:tt)*
    ) => {
        messages!(
            $( #[$attr] )*
            $selector => $name (
                &mut self $( , $param_name : $param_ty )*
            ) -> ();

            $($rest)*
        );
    };
    // Base rule to end macro.
    () => {};
}
