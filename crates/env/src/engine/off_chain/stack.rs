use std::vec::Vec;
use std::collections::BTreeMap;

use crate::{AccountId, Hash, account_id, ReturnFlags};


/// A frame in the call stack
#[derive(Clone, Debug)]
pub struct Frame {
    pub level: u32,
    pub caller: Option<AccountId>,
    pub callee: AccountId,
    pub input: Vec<u8>,
    pub return_value: Option<(ReturnFlags, Vec<u8>)>
}

pub struct Stack {
    pub stack: Vec<Frame>,
}

impl Stack {
    /// Crates a call stack with the default `account`
    pub fn new(account: AccountId) -> Self {
        Self {
            stack: vec![Frame {
                level: 0,
                caller: None,
                callee: account,
                input: Default::default(),
                return_value: None,
            }],
        }
    }

    /// Changes the caller account
    ///
    /// Only allowed outside any contract call (when the stack is empty).
    pub fn switch_account(&mut self, account: AccountId) -> Result<(), ()> {
        let stack = &mut self.stack;
        if stack.len() != 1 {
            return Err(())
        }
        let ctx = stack.get_mut(0).ok_or(())?;
        ctx.callee = account;
        Ok(())
    }


    /// Pushes a new call frame
    pub fn push(&mut self, callee: &AccountId, input: Vec<u8>) {
        let parent_ctx = self.peek();
        self.stack.push(Frame {
            level: parent_ctx.level + 1,
            caller: Some(parent_ctx.callee),
            callee: callee.clone(),
            input,
            return_value: None,
        });
        self.sync_to_ink();
    }

    /// Pops the call frame and returns the frame
    pub fn pop(&mut self) -> Option<Frame> {
        if self.stack.len() > 1 {
            let ctx = self.stack.pop();
            self.sync_to_ink();
            ctx
        } else {
            None
        }
    }

    /// Peeks the current call frame
    pub fn peek(&self) -> Frame {
        self.stack.last().cloned().expect("stack is never empty; qed.")
    }

    pub fn set_return_value(&mut self, flags: ReturnFlags, value: Vec<u8>) {
        let cur = self.stack.last_mut().expect("stack is never empty; qed.");
        cur.return_value = Some((flags, value));
    }

    pub fn origin(&self) -> AccountId {
        self.stack.first().expect("stack is never empty; qed").callee
    }

    /// Syncs the top call frame to ink testing environment
    pub fn sync_to_ink(&self) {}
}

#[derive(Default)]
pub struct ContractStore {
    code: BTreeMap<AccountId, Hash>,
    fns: BTreeMap<Hash, (fn(), fn())>,
}

impl ContractStore {
    pub fn register_contract(&mut self, code: Hash, id: AccountId) {
        self.code.insert(id, code);
    }
    pub fn register_entrypoints(&mut self, code: Hash, deploy: fn(), call: fn())
    {
        self.fns.insert(code, (deploy, call));
    }
    pub fn entrypoints(&self, account: &AccountId) -> Option<(fn(), fn())> {
        let code = self.code.get(account)?;
        self.fns.get(code).cloned()
    }
    pub fn is_contract(&self, account: &AccountId) -> bool {
        self.code.contains_key(account)
    }
    pub fn code_hash(&self, account: &AccountId) -> Option<Hash> {
        self.code.get(account).cloned()
    }
    pub fn update_code(&mut self, account: AccountId, code: Hash) {
        // account can be non-existing?
        self.code.insert(account, code);
    }
    pub fn next_address_of(&self, code: &Hash) -> AccountId {
        let count = self.code
            .iter()
            .filter(|(_k, v)| *v == code)
            .count();
        let mut raw = [0u8; 32];
        raw.copy_from_slice(code.as_ref());
        raw[raw.len() - 1] = count as u8;
        AccountId::from(raw)
    }
}
