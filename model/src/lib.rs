#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_fn)]

#![allow(unused)]

#[macro_use]
mod state;

#[macro_use]
mod msg;

mod contract;
mod exec_env;
mod msg_handler;
