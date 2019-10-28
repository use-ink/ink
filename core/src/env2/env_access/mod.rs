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

mod immutable;
mod mutable;

pub use self::{
    immutable::EnvAccess,
    mutable::{
        EmitEvent,
        EnvAccessMut,
    },
};

/// Allows to access the environment from `&EnvAccess` and `&mut EnvAccess`
/// respectively with different degree of efficiency.
pub trait AccessEnv {
    /// The environment definition.
    type Target;

    /// Access the environment.
    fn env(self) -> Self::Target;
}
