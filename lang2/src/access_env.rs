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

use ink_core::env2::{
    DynEnv,
    EnvAccess,
    EnvAccessMut,
};

/// Allows to directly access the environment read-only.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait AccessEnv {
    /// The environment accessor.
    ///
    /// # Note
    ///
    /// This can be any of `ink_core::env::DynEnv` or `ink_core::env::EnvAccessMut`.
    /// The set of possible types may be extended in the future.
    type Target;

    /// Returns an immutable access to the environment.
    fn env(&self) -> &Self::Target;
}

/// Allows to directly access the environment mutably.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait AccessEnvMut: AccessEnv {
    /// Returns a mutable access to the environment.
    fn env_mut(&mut self) -> &mut Self::Target;
}

impl<E> AccessEnv for DynEnv<E> {
    type Target = E;

    fn env(&self) -> &Self::Target {
        DynEnv::env(self)
    }
}

impl<E> AccessEnvMut for DynEnv<E> {
    fn env_mut(&mut self) -> &mut Self::Target {
        DynEnv::env_mut(self)
    }
}

impl<E> AccessEnv for EnvAccess<E> {
    type Target = Self;

    fn env(&self) -> &Self::Target {
        self
    }
}

impl<E> AccessEnv for EnvAccessMut<E> {
    type Target = Self;

    fn env(&self) -> &Self::Target {
        self
    }
}

impl<E> AccessEnvMut for EnvAccessMut<E> {
    fn env_mut(&mut self) -> &mut Self::Target {
        self
    }
}