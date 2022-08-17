// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

mod call_data;
mod impls;
pub mod test_api;
mod types;
mod stack;

#[cfg(test)]
mod tests;

pub use call_data::CallData;

use super::OnInstance;
use crate::Error;
use stack::{Stack, Frame, ContractStore};

use derive_more::From;
use ink_engine::ext::Engine;

/// The off-chain environment.
pub struct EnvInstance {
    engine: Engine,
    stack: Stack,
    pub contracts: ContractStore,
}

impl EnvInstance {
    fn sync_stack(&mut self) {
        use scale::Encode;

        let ctx = self.stack.peek();
        if let Some(caller) = ctx.caller {
            self.engine.set_caller(caller.encode());
        }
        self.engine.set_callee(ctx.callee.encode());
    }

    fn push_frame(&mut self, callee: &crate::AccountId, input: Vec<u8>) {
        self.stack.push(callee, input);
        self.sync_stack();
    }

    fn pop_frame(&mut self) -> Option<Frame> {
        let ctx = self.stack.pop();
        if ctx.is_some() {
            self.sync_stack();
        }
        ctx
    }

    pub fn call<R: scale::Decode>(&mut self, callee: &crate::AccountId, input: Vec<u8>) -> crate::Result<R> {
        self.push_frame(callee, input.clone());
        let (_deploy, call) = self.contracts.entrypoints(callee)
            .ok_or(Error::NotCallable)?;
        // TODO: snapshot the db
        // TODO: unwind panic?
        call();
        // Read return value & process revert
        let frame = self.pop_frame().expect("frame exists; qed.");
        let data = if let Some((flags, data)) = frame.return_value {
            if flags.reverted() {
                // TODO: revert the db snapshot
                return Err(Error::CalleeReverted)
            }
            data
        } else {
            Default::default()
        };
        scale::Decode::decode(&mut &data[..])
            .map_err(|err| Error::Decode(err))
    }

    pub fn deploy(&mut self, account: &crate::AccountId, input: Vec<u8>) -> crate::Result<()> {
        self.push_frame(account, input.clone());
        let (deploy, _call) = self.contracts.entrypoints(account)
            .ok_or(Error::NotCallable)?;
        deploy();
        self.pop_frame();
        // Read OUTPUT
        // what if revert?
        // scale::Decode::decode(&mut &input[..])
        //     .map_err(|err| Error::Decode(err))
        Ok(())
    }

    pub fn caller_is_origin(&self) -> bool {
        let origin = self.stack.origin();
        let ctx = self.stack.peek();
        assert!(ctx.level > 0, "should never reach when there's no running contract");
        ctx.caller.expect("contract has caller; qed.") == origin
    }

}

impl OnInstance for EnvInstance {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        use core::cell::RefCell;
        thread_local!(
            static INSTANCE: RefCell<EnvInstance> = RefCell::new(
                EnvInstance {
                    engine: Engine::new(),
                    stack: Stack::new(crate::AccountId::from([1u8; 32])),
                    contracts: Default::default(),
                }
            )
        );
        INSTANCE.with(|instance| f(&mut instance.borrow_mut()))
    }
}

#[derive(Debug, From, PartialEq, Eq)]
pub enum OffChainError {
    Account(AccountError),
    #[from(ignore)]
    UninitializedBlocks,
    #[from(ignore)]
    UninitializedExecutionContext,
    #[from(ignore)]
    UnregisteredChainExtension,
}

/// Errors encountered upon interacting with the accounts database.
#[derive(Debug, From, PartialEq, Eq)]
pub enum AccountError {
    Decoding(scale::Error),
    #[from(ignore)]
    UnexpectedUserAccount,
    #[from(ignore)]
    NoAccountForId(Vec<u8>),
}
