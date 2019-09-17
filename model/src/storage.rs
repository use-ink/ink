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
    EnvAccess,
    Named,
};
use ink_core::storage::{
    alloc::{
        AllocateUsing,
        Initialize,
    },
    Flush,
};

/// Types implementing this type can be used as contract state.
pub trait Storage: AllocateUsing + Initialize + Flush + Named + EnvAccess {}

/// Define contract state with less boilerplate code.
#[macro_export]
macro_rules! storage {
	(
		$( #[$state_meta:meta] )*
		$vis:vis struct $state_name:ident {
			$(
				$( #[$field_meta:meta] )*
				$field_name:ident : $field_ty:ty ,
			)*
		}
	) => {
		$( #[$state_meta] )*
		$vis struct $state_name<E> {
			$(
				$( #[$field_meta] )*
				$field_name : $field_ty ,
			)*
            env: $crate::EnvHandler<E>,
		}

		impl<E> ink_core::storage::Flush for $state_name<E> {
			fn flush(&mut self) {
				$(
					self.$field_name.flush();
				)*
                self.env.flush();
			}
		}

		impl<E> ink_core::storage::alloc::AllocateUsing for $state_name<E> {
			unsafe fn allocate_using<A>(alloc: &mut A) -> Self
			where
				A: ink_core::storage::alloc::Allocate,
			{
				use ink_core::storage::alloc::AllocateUsing;
				Self {
					$(
						$field_name : AllocateUsing::allocate_using(alloc),
					)*
                    env: AllocateUsing::allocate_using(alloc),
				}
			}
		}

        impl<E> ink_core::storage::alloc::Initialize for $state_name<E> {
            type Args = ();

            #[inline(always)]
            fn default_value() -> Option<Self::Args> {
                // With this we can also default initialize storage state structs.
                Some(())
            }

            fn initialize(&mut self, args: Self::Args) {
                $(
                    self.$field_name.try_default_initialize();
                )*
                self.env.try_default_initialize();
            }
        }

        impl<E> $crate::EnvAccess for $state_name<E> {
            type Env = $crate::EnvHandler<E>;

            fn env(&self) -> &Self::Env {
                &self.env
            }

            fn env_mut(&mut self) -> &mut Self::Env {
                &mut self.env
            }
        }

        impl<E> $crate::Named for $state_name<E> {
			const NAME: &'static str = stringify!($state_name);
        }

		impl<E> $crate::Storage for $state_name<E> {}
	};
	(
		$( #[$state_meta:meta] )*
		$vis:vis struct $state_name:ident {
			$(
				$( #[$field_meta:meta] )*
				$field_name:ident : $field_ty:ty
			),*
		}
	) => {
		$crate::storage! {
			$vis struct $state_name {
				$(
					$( #[$field_meta] )*
					$field_name : $field_ty ,
				)*
			}
		}
	};
}
