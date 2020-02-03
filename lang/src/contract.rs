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

use crate::{
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
use ink_core::env::EnvTypes;

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
        use ink_core::storage::alloc::{
            AllocateUsing,
            BumpAlloc,
        };
        use ink_primitives::Key;
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
    pub fn dispatch_using_mode<T>(
        mut self,
        mode: DispatchMode,
    ) -> Result<(), DispatchError>
    where
        T: EnvTypes,
    {
        // Initialize storage if we instantiate the contract.
        if mode == DispatchMode::Instantiate {
            self.storage.try_default_initialize();
        }
        // Dispatch using the contract execution input.
        let call_data =
            ink_core::env::input().map_err(|_| DispatchError::CouldNotReadInput)?;
        match mode {
            DispatchMode::Instantiate => {
                self.constructors
                    .dispatch::<T>(&mut self.storage, &call_data)
            }
            DispatchMode::Call => {
                self.messages.dispatch::<T>(&mut self.storage, &call_data)
            }
        }
    }
}

/// Trait implemented by contracts themselves in order to provide a clean
/// interface for the C-ABI specified `call` and `create` functions to forward
/// calls to.
pub trait DispatchUsingMode {
    fn dispatch_using_mode(mode: DispatchMode) -> Result<(), DispatchError>;
}
