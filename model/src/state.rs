use pdsl_core::{
	storage::{
		Flush,
		alloc::{
			AllocateUsing,
			Initialize,
		},
	},
};

/// Types implementing this type can be used as contract state.
pub trait ContractState:
	AllocateUsing + Flush
{
	/// The name of the contract state.
	///
	/// # Note
	///
	/// - This should be a valid Rust identifier.
	/// - Normally you simply want to use the name of the contract here.
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
				$field_name:ident : $field_ty:ty
			),*
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
					self.$field_name.flush();
				),*
			}
		}

		impl pdsl_core::storage::alloc::AllocateUsing for $state_name {
			unsafe fn allocate_using<A>(alloc: &mut A) -> Self
			where
				A: pdsl_core::storage::Allocator,
			{
				use pdsl_core::storage::alloc::AllocateUsing;
				Self {
					$(
						$field_name : AllocateUsing::allocate_using(alloc)
					),*
				}
			}
		}

		impl pdsl_model::state::ContractState for $state_name {
			const NAME: &'static str = stringify!($state_name);
		}
	};
}

use crate as pdsl_model; // TODO: Fix this to being not required!

state! {
	/// An empty contract state.
	pub struct EmptyContractState {}
}
