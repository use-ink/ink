#![cfg_attr(not(feature = "std"), no_std)]
#![feature(
	const_fn,
	const_str_as_bytes,
	core_intrinsics,
)]
#![deny(bad_style,
	const_err,
	dead_code,
	improper_ctypes,
	legacy_directory_ownership,
	non_shorthand_field_patterns,
	no_mangle_generic_items,
	overflowing_literals,
	path_statements,
	patterns_in_fns_without_body,
	plugin_as_library,
	private_in_public,
	safe_extern_statics,
	unconditional_recursion,
	unions_with_drop_fields,
	unused,
	unused_allocation,
	unused_comparisons,
	unused_parens,
	while_true,
	// missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unused_extern_crates,
	// unused_import_braces,
	unused_qualifications,
	unused_results,
	// missing-copy-implementations
)]

#[macro_use]
mod state;

#[macro_use]
mod msg;

mod contract;
mod exec_env;
mod msg_handler;

mod test;

pub use crate::{
	state::{
		ContractState,
	},
	contract::{
		EmptyContractState,
		NoDeployArgs,
		DeployHandler,
		ContractDecl,
		ContractInstance,
		Contract,
	},
	msg::{
		Message,
	},
	exec_env::{
		ExecutionEnv,
	},
	msg_handler::{
		CallData,
		Error,
		MessageHandlerSelector,
		MessageHandler,
		MessageHandlerMut,
		RawMessageHandler,
		RawMessageHandlerMut,
		Result,
		HandleCall,
		UnreachableMessageHandler,
	},
};
