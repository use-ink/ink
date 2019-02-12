use pdsl_core::{
	storage::{
		Flush,
		alloc::{
			AllocateUsing,
			Initialize,
		},
	},
};

pub trait ContractState:
	AllocateUsing + Initialize + Flush {}

#[derive(Copy, Clone)]
pub struct EmptyContractState;

impl ContractState for EmptyContractState {}

impl AllocateUsing for EmptyContractState {
	unsafe fn allocate_using<A>(_alloc: &mut A) -> Self {
		EmptyContractState
	}
}

impl Initialize for EmptyContractState {
	type Args = ();

	fn initialize(&mut self, _args: Self::Args) {}
}

impl Flush for EmptyContractState {
	fn flush(&mut self) {}
}

#[macro_export]
macro_rules! state {
	(
		$state_name:ident {
			$(
				$field_name:ident : $field_ty:ty
			),*
		}
	) => {
		struct $state_name {
			$(
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

		impl ContractState for $state_name {}
	};
}
