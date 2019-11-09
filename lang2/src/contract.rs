// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    AccessEnv,
    Dispatch,
    DispatchError,
    DispatchList,
    DispatchableFn,
    DispatchableFnMut,
    Dispatcher,
    DispatcherMut,
    EmptyDispatchList,
    FnInput,
    FnOutput,
    PushDispatcher,
};
use core::{
    marker::PhantomData,
    mem::ManuallyDrop,
};

/// The contract definition.
pub struct Contract<Storage, Constrs, Msgs> {
    /// The storage holding contract state.
    pub storage: ManuallyDrop<Storage>,
    /// The dispatchable constructors.
    pub constructors: Constrs,
    /// The dispatchable messages.
    pub messages: Msgs,
}

impl Contract<(), (), ()> {
    /// Creates a new contract definition for the given storage type.
    pub fn with_storage<Storage>(
    ) -> ContractBuilder<Storage, EmptyDispatchList, EmptyDispatchList> {
        ContractBuilder {
            storage: Default::default(),
            constructors: DispatchList::empty(),
            messages: DispatchList::empty(),
        }
    }
}

/// Storage marker.
#[derive(Debug, Copy, Clone)]
struct StorageMarker<S>(PhantomData<fn() -> S>);

impl<S> Default for StorageMarker<S> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Simplifies declaration of a smart contract.
pub struct ContractBuilder<Storage, Constrs, Msgs> {
    storage: StorageMarker<Storage>,
    constructors: Constrs,
    messages: Msgs,
}

impl<Storage, Constrs> ContractBuilder<Storage, Constrs, EmptyDispatchList>
where
    Constrs: PushDispatcher,
{
    /// Pushes a new constructor to the contract definition.
    pub fn on_instantiate<C>(
        self,
        dfn: DispatchableFnMut<C, Storage>,
    ) -> ContractBuilder<
        Storage,
        DispatchList<DispatcherMut<C, Storage>, Constrs>,
        EmptyDispatchList,
    >
    where
        C: FnInput + FnOutput,
    {
        ContractBuilder {
            storage: self.storage,
            constructors: self.constructors.push(DispatcherMut::new(dfn)),
            messages: self.messages,
        }
    }
}

impl<Storage, Constrs, Msgs> ContractBuilder<Storage, Constrs, Msgs>
where
    Msgs: PushDispatcher,
{
    /// Pushes a new message to the contract definition.
    ///
    /// The message may not mutate contract storage.
    pub fn on_msg<M>(
        self,
        dfn: DispatchableFn<M, Storage>,
    ) -> ContractBuilder<Storage, Constrs, DispatchList<Dispatcher<M, Storage>, Msgs>>
    where
        M: FnInput + FnOutput,
    {
        ContractBuilder {
            storage: self.storage,
            constructors: self.constructors,
            messages: self.messages.push(Dispatcher::new(dfn)),
        }
    }
}

impl<Storage, Constrs, Msgs> ContractBuilder<Storage, Constrs, Msgs>
where
    Msgs: PushDispatcher,
{
    /// Pushes a new message to the contract definition.
    ///
    /// The message may not mutate contract storage.
    pub fn on_msg_mut<M>(
        self,
        dfn: DispatchableFnMut<M, Storage>,
    ) -> ContractBuilder<Storage, Constrs, DispatchList<DispatcherMut<M, Storage>, Msgs>>
    where
        M: FnInput + FnOutput,
    {
        ContractBuilder {
            storage: self.storage,
            constructors: self.constructors,
            messages: self.messages.push(DispatcherMut::new(dfn)),
        }
    }
}

impl<Storage, ConstrsHead, ConstrsRest, MsgsHead, MsgsRest>
    ContractBuilder<
        Storage,
        DispatchList<ConstrsHead, ConstrsRest>,
        DispatchList<MsgsHead, MsgsRest>,
    >
{
    /// Finalizes construction of the contract definition.
    pub fn done(
        self,
    ) -> Contract<
        Storage,
        DispatchList<ConstrsHead, ConstrsRest>,
        DispatchList<MsgsHead, MsgsRest>,
    >
    where
        Storage: crate::Storage,
    {
        use ink_core::storage::{
            alloc::{
                AllocateUsing,
                BumpAlloc,
            },
            Key,
        };
        let storage = ManuallyDrop::new(unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            AllocateUsing::allocate_using(&mut alloc)
        });
        Contract {
            storage,
            constructors: self.constructors,
            messages: self.messages,
        }
    }
}

/// The contract dispatch mode.
///
/// Tells the [`Contract::dispatch_using_mode`] routine what to dispatch for.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DispatchMode {
    /// Mode for instantiating a contract.
    Instantiate,
    /// Mode for calling a contract.
    Call,
}

impl<Storage, ConstrsHead, ConstrsRest, MsgsHead, MsgsRest>
    Contract<
        Storage,
        DispatchList<ConstrsHead, ConstrsRest>,
        DispatchList<MsgsHead, MsgsRest>,
    >
where
    Storage: crate::Storage,
    DispatchList<ConstrsHead, ConstrsRest>: Dispatch<Storage>,
    DispatchList<MsgsHead, MsgsRest>: Dispatch<Storage>,
{
    pub fn dispatch_using_mode<Env>(
        mut self,
        mode: DispatchMode,
    ) -> Result<(), DispatchError>
    where
        Storage: AccessEnv<Env>,
        Env: ink_core::env2::Env,
    {
        // Initialize storage if we instantiate the contract.
        if mode == DispatchMode::Instantiate {
            self.storage.try_default_initialize();
        }
        // Dispatch using the contract execution input.
        let call_data = self.storage.access_env().input();
        let ret = match mode {
            DispatchMode::Instantiate => {
                self.constructors.dispatch(&mut self.storage, &call_data)
            }
            DispatchMode::Call => self.messages.dispatch(&mut self.storage, &call_data),
        };
        ret.into()
    }
}

/// Trait implemented by contracts themselves in order to provide a clean
/// interface for the C-ABI specified `call` and `create` functions to forward
/// calls to.
pub trait DispatchUsingMode {
    fn dispatch_using_mode(mode: DispatchMode) -> Result<(), DispatchError>;
}
