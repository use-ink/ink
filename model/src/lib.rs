#![cfg_attr(not(feature = "std"), no_std)]
#![feature(
	const_fn,
	const_str_as_bytes,
)]
#![allow(unused)]

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
		NoDeployArgs,
		DeployHandler,
		ContractDecl,
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
