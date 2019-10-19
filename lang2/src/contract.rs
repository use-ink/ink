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
    AccessEnvMut,
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
    Storage,
};
use core::{
    marker::PhantomData,
    mem::ManuallyDrop,
};
use ink_core::env2::{
    EnvAccess,
    EnvAccessMut,
};

/// A storage type pair.
///
/// The `Mut` part is the main storage with an `EnvAccessMut` or `DynEnv<EnvAccessMut>`
/// environmental definition. The `Imm` (a.k.a. immutable or read-only) part must be trivially
/// constructible from the `Mut` version which is true for `EnvAccess` or `DynEnv<EnvAccess>`
/// respectively.
pub trait StoragePair {
    /// The original storage definition.
    type Mut: Storage;
    /// The immutable (read-only) storage definition.
    type Imm: From<Self::Mut>;
}

impl<SMut, SImm> StoragePair for (SMut, SImm)
where
    SMut: Storage,
    SImm: From<SMut>,
{
    type Mut = SMut;
    type Imm = SImm;
}

pub struct StoragePlaceholder;
pub struct StoragePlaceholderMut;
pub struct StoragePlaceholderImm;

impl From<StoragePlaceholderMut> for StoragePlaceholderImm {
    fn from(_mutable: StoragePlaceholderMut) -> Self {
        StoragePlaceholderImm
    }
}

impl StoragePair for StoragePlaceholder {
    type Mut = StoragePlaceholderMut;
    type Imm = StoragePlaceholderImm;
}

impl ink_core::storage::alloc::AllocateUsing for StoragePlaceholderMut {
    unsafe fn allocate_using<A>(_alloc: &mut A) -> Self
    where
        A: ink_core::storage::alloc::Allocate,
    {
        Self
    }
}

impl ink_core::storage::alloc::Initialize for StoragePlaceholderMut {
    type Args = ();

    fn initialize(&mut self, _: Self::Args) {}
}

impl ink_core::storage::Flush for StoragePlaceholderMut {
    fn flush(&mut self) {}
}

impl Storage for StoragePlaceholderMut {}

pub struct StorageMutImm<S>
where
    S: StoragePair,
{
    pub storage_mut: ManuallyDrop<<S as StoragePair>::Mut>,
    storage_imm: PhantomData<fn () -> <S as StoragePair>::Imm>,
}

impl<S> StorageMutImm<S>
where
    S: StoragePair,
{
    pub fn allocate() -> Self {
        use ink_core::storage::{
            alloc::{
                AllocateUsing,
                BumpAlloc,
            },
            Key,
        };
        let storage: <S as StoragePair>::Mut = unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            AllocateUsing::allocate_using(&mut alloc)
        };
        Self {
            storage_mut: ManuallyDrop::new(storage),
            storage_imm: Default::default(),
        }
    }

    pub fn into_imm(self) -> ManuallyDrop<<S as StoragePair>::Imm> {
        ManuallyDrop::new(
            ManuallyDrop::into_inner(self.storage_mut).into()
        )
    }
}

impl<S> core::ops::Deref for StorageMutImm<S>
where
    S: StoragePair,
{
    type Target = <S as StoragePair>::Mut;

    fn deref(&self) -> &Self::Target {
        &self.storage_mut
    }
}

impl<S> core::ops::DerefMut for StorageMutImm<S>
where
    S: StoragePair,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage_mut
    }
}

/// The contract definition.
pub struct Contract<Storage, Constrs, Msgs, MutMsgs>
where
    Storage: StoragePair,
{
    /// The storage holding contract state.
    pub storage: StorageMutImm<Storage>,
    /// The dispatchable constructors.
    pub constructors: Constrs,
    /// The dispatchable messages that may not mutate storage.
    pub messages: Msgs,
    /// The dispatchable messages that may mutate storage.
    pub mut_messages: MutMsgs,
}

impl Contract<StoragePlaceholder, (), (), ()> {
    /// Creates a new contract definition for the given storage type.
    pub fn with_storage<Storage>() -> ContractBuilder<
        (<Storage as StoragePair>::Mut, <Storage as StoragePair>::Imm),
        EmptyDispatchList,
        EmptyDispatchList,
        EmptyDispatchList,
    >
    where
        Storage: StoragePair,
    {
        ContractBuilder {
            storage: Default::default(),
            constructors: DispatchList::empty(),
            messages: DispatchList::empty(),
            mut_messages: DispatchList::empty(),
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
pub struct ContractBuilder<
    Storage,
    Constrs = EmptyDispatchList,
    Msgs = EmptyDispatchList,
    MutMsgs = EmptyDispatchList,
> {
    storage: StorageMarker<Storage>,
    constructors: Constrs,
    messages: Msgs,
    mut_messages: MutMsgs,
}

impl<Storage, Constrs> ContractBuilder<Storage, Constrs>
where
    Storage: StoragePair,
    Constrs: PushDispatcher,
{
    /// Pushes a new constructor to the contract definition.
    pub fn on_instantiate<C>(
        self,
        dfn: DispatchableFnMut<C, <Storage as StoragePair>::Mut>,
    ) -> ContractBuilder<Storage, DispatchList<DispatcherMut<C, <Storage as StoragePair>::Mut>, Constrs>>
    where
        C: FnInput + FnOutput,
    {
        ContractBuilder {
            storage: self.storage,
            constructors: self.constructors.push(DispatcherMut::new(dfn)),
            messages: self.messages,
            mut_messages: self.mut_messages,
        }
    }
}

impl<Storage, Constrs, Msgs, MutMsgs> ContractBuilder<Storage, Constrs, Msgs, MutMsgs>
where
    Storage: StoragePair,
    Msgs: PushDispatcher,
{
    /// Pushes a new message to the contract definition.
    ///
    /// The message may not mutate contract storage.
    pub fn on_msg<M>(
        self,
        dfn: DispatchableFn<M, <Storage as StoragePair>::Imm>,
    ) -> ContractBuilder<
        Storage,
        Constrs,
        DispatchList<Dispatcher<M, <Storage as StoragePair>::Imm>, Msgs>,
        MutMsgs,
    >
    where
        M: FnInput + FnOutput,
    {
        ContractBuilder {
            storage: self.storage,
            constructors: self.constructors,
            messages: self.messages.push(Dispatcher::new(dfn)),
            mut_messages: self.mut_messages,
        }
    }
}

impl<Storage, Constrs, Msgs, MutMsgs> ContractBuilder<Storage, Constrs, Msgs, MutMsgs>
where
    Storage: StoragePair,
    MutMsgs: PushDispatcher,
{
    /// Pushes a new message to the contract definition.
    ///
    /// The message may not mutate contract storage.
    pub fn on_msg_mut<M>(
        self,
        dfn: DispatchableFnMut<M, <Storage as StoragePair>::Mut>,
    ) -> ContractBuilder<
        Storage,
        Constrs,
        Msgs,
        DispatchList<DispatcherMut<M, <Storage as StoragePair>::Mut>, MutMsgs>,
    >
    where
        M: FnInput + FnOutput,
    {
        ContractBuilder {
            storage: self.storage,
            constructors: self.constructors,
            messages: self.messages,
            mut_messages: self.mut_messages.push(DispatcherMut::new(dfn)),
        }
    }
}

impl<Storage, ConstrsHead, ConstrsRest, MsgsHead, MsgsRest, MutMsgsHead, MutMsgsRest>
    ContractBuilder<
        Storage,
        DispatchList<ConstrsHead, ConstrsRest>,
        DispatchList<MsgsHead, MsgsRest>,
        DispatchList<MutMsgsHead, MutMsgsRest>,
    >
where
    Storage: StoragePair,
{
    /// Finalizes construction of the contract definition.
    pub fn done(
        self,
    ) -> Contract<
        Storage,
        DispatchList<ConstrsHead, ConstrsRest>,
        DispatchList<MsgsHead, MsgsRest>,
        DispatchList<MutMsgsHead, MutMsgsRest>,
    > {
        Contract {
            storage: StorageMutImm::allocate(),
            constructors: self.constructors,
            messages: self.messages,
            mut_messages: self.mut_messages,
        }
    }
}

/// Interface to work with instantiated contracts.
pub trait ContractInstance {
    /// Instantiates or calls the contract.
    fn dispatch(self, mode: DispatchMode) -> Result<(), DispatchError>;
}

/// The contract dispatch mode.
///
/// Tells the [`Contract::dispatch`] routine what to dispatch for.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DispatchMode {
    /// Mode for instantiating a contract.
    Instantiate,
    /// Mode for calling a contract.
    Call,
}

impl<
        Storage,
        ConstrsHead,
        ConstrsRest,
        MsgsHead,
        MsgsRest,
        MutMsgsHead,
        MutMsgsRest,
        Env,
    >
    Contract<
        Storage,
        DispatchList<ConstrsHead, ConstrsRest>,
        DispatchList<MsgsHead, MsgsRest>,
        DispatchList<MutMsgsHead, MutMsgsRest>,
    >
where
    Storage: StoragePair,
    <Storage as StoragePair>::Mut: crate::Storage + AccessEnvMut,
    <<Storage as StoragePair>::Mut as AccessEnv>::Target: AccessEnvMut<Target = EnvAccessMut<Env>>,
    <Storage as StoragePair>::Imm: AccessEnv + From<<Storage as StoragePair>::Mut>,
    <<Storage as StoragePair>::Imm as AccessEnv>::Target: AccessEnv<Target = EnvAccess<Env>>,
    DispatchList<ConstrsHead, ConstrsRest>: Dispatch<<Storage as StoragePair>::Mut>,
    DispatchList<MsgsHead, MsgsRest>: Dispatch<<Storage as StoragePair>::Imm>,
    DispatchList<MutMsgsHead, MutMsgsRest>: Dispatch<<Storage as StoragePair>::Mut>,
    Env: ink_core::env2::Env,
{
    pub fn dispatch_using_mode(mut self, mode: DispatchMode) -> Result<(), DispatchError> {
        // Initialize storage if we instantiate the contract.
        if mode == DispatchMode::Instantiate {
            use ink_core::storage::alloc::Initialize as _;
            self.storage.try_default_initialize();
        }
        // Dispatch using the contract execution input.
        let call_data = self.storage.env_mut().env_mut().input();
        let ret = match mode {
            DispatchMode::Instantiate => {
                self.constructors.dispatch(&mut self.storage, &call_data)
            }
            DispatchMode::Call => {
                // Try to dispatch into mutable messages first.
                // If this returns an error we then convert the storage into its
                // immutable (read-only) version and try to dispatch for messages
                // that may not mutate the contract storage.
                let ret = self.mut_messages.dispatch(&mut self.storage, &call_data);
                if ret.is_ok() {
                    ret
                } else {
                    self.messages.dispatch(&mut self.storage.into_imm(), &call_data)
                }
            }
        };
        ret.into()
    }
}

pub trait ContractDispatch {
    fn dispatch(mode: DispatchMode) -> Result<(), DispatchError>;
}
