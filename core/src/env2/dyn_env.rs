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
    env2::{
        EnvAccess,
        EnvAccessMut,
    },
    storage::{
        alloc::{
            Allocate,
            AllocateUsing,
            DynAlloc,
            Initialize,
        },
        Flush,
    },
};
use core::ops::{
    Deref,
    DerefMut,
};

/// Environment with `&self` access and a dynamic allocator.
///
/// # Note
///
/// - Accesses to `DynEnvAccess` are checked at runtime.
/// - The dynamic allocator allows to dynamically allocate and deallocate objects on the storage.
pub type DynEnvAccess<E> = DynEnv<EnvAccess<E>>;

/// Environment with `&mut self`-only access and a dynamic allocator.
///
/// # Note
///
/// - Accesses to `DynEnvAccessMut` are checked at compiletime.
/// - The dynamic allocator allows to dynamically allocate and deallocate objects on the storage.
pub type DynEnvAccessMut<E> = DynEnv<EnvAccessMut<E>>;

/// A wrapper around `EnvAccess` or `EnvAccessMut` that adds a dynamic storage allocator.
pub struct DynEnv<E> {
    /// The wrapped environment.
    env: E,
    /// The dynamic storage allocator.
    alloc: DynAlloc,
}

impl<E> DynEnv<E> {
    #[inline]
    pub fn env(&self) -> &E {
        &self.env
    }

    #[inline]
    pub fn env_mut(&mut self) -> &mut E {
        &mut self.env
    }
}

impl<E> From<DynEnv<EnvAccessMut<E>>> for DynEnv<EnvAccess<E>> {
    fn from(dyn_env: DynEnv<EnvAccessMut<E>>) -> Self {
        Self {
            env: dyn_env.env.into(),
            alloc: dyn_env.alloc,
        }
    }
}

impl<E> Deref for DynEnv<E> {
    type Target = E;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.env
    }
}

impl<E> DerefMut for DynEnv<E> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.env
    }
}

impl<E> Flush for DynEnv<E> {
    #[inline]
    fn flush(&mut self) {
        self.alloc.flush()
    }
}

impl<E> AllocateUsing for DynEnv<E>
where
    E: Default,
{
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            env: Default::default(),
            alloc: AllocateUsing::allocate_using(alloc),
        }
    }
}

impl<E> Initialize for DynEnv<E> {
    type Args = ();

    #[inline]
    fn initialize(&mut self, _args: Self::Args) {
        self.alloc.initialize(());
    }
}
