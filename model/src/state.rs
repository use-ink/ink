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

use ink_core::storage::{
    alloc::{
        AllocateUsing,
        Initialize,
    },
    Flush,
};

/// Types implementing this type can be used as contract state.
pub trait ContractState: AllocateUsing + Initialize + Flush {
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

		impl ink_core::storage::Flush for $state_name {
			fn flush(&mut self) {
				$(
					self.$field_name.flush()
				);*
			}
		}

		impl ink_core::storage::alloc::AllocateUsing for $state_name {
			unsafe fn allocate_using<A>(alloc: &mut A) -> Self
			where
				A: ink_core::storage::alloc::Allocate,
			{
				use ink_core::storage::alloc::AllocateUsing;
				Self {
					$(
						$field_name : AllocateUsing::allocate_using(alloc)
					),*
				}
			}
		}

        impl ink_core::storage::alloc::Initialize for $state_name {
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
