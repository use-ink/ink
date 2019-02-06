#![no_std]

// use pdsl_core::{
// 	env::{Env, ContractEnv},
// };

#[allow(unused)]
use pdsl_core;

#[no_mangle]
pub extern "C" fn deploy() {
	// ContractEnv::println("noop contract: CREATE");
}

#[no_mangle]
pub extern "C" fn call() {
	// ContractEnv::println("noop contract: CALL");
}
