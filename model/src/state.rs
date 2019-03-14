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

use pdsl_core::storage::{
    alloc::AllocateUsing,
    Flush,
};

/// Types implementing this type can be used as contract state.
pub trait ContractState: AllocateUsing + Flush {
    /// The name of the contract state.
    ///
    /// # Note
    ///
    /// - This must be a valid Rust identifier.
    /// - Normally this reflects the name of the contract.
    // const NAME: &'static str;
    const NAME: &'static str;
}

/// Define contract state with less boilerplate code.
#[macro_export]
macro_rules! state {
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
		$vis struct $state_name {
			$(
				$( #[$field_meta] )*
				$field_name : $field_ty
			),*
		}

		impl pdsl_core::storage::Flush for $state_name {
			fn flush(&mut self) {
				$(
					self.$field_name.flush()
				);*
			}
		}

		impl pdsl_core::storage::alloc::AllocateUsing for $state_name {
			unsafe fn allocate_using<A>(alloc: &mut A) -> Self
			where
				A: pdsl_core::storage::alloc::Allocate,
			{
				use pdsl_core::storage::alloc::AllocateUsing;
				Self {
					$(
						$field_name : AllocateUsing::allocate_using(alloc)
					),*
				}
			}
		}

		impl $crate::ContractState for $state_name {
			const NAME: &'static str = stringify!($state_name);
		}
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
		$crate::state! {
			$vis struct $state_name {
				$(
					$( #[$field_meta] )*
					$field_name : $field_ty ,
				)*
			}
		}
	};
}
