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

//! Compile time checks.

use crate::Message;

/// Shortcut for `true` array type.
pub type True<T = ()> = [T; true as usize];
/// Shortcut for `false` array type.
pub type False<T = ()> = [T; false as usize];

/// Used to check at compile-time if a message is allowed to mutate state.
pub trait CheckIsMessageMut: Message {
    type Value: IsMessageMutRename;
}

/// Used to rename associated types to improve the displayed error messages.
pub trait IsMessageMutRename {
    type Value;
}
/// Rename a true type simply to the truth type for `IsMutMessage`.
impl<Msg> IsMessageMutRename for True<Msg> {
    type Value = MessageIsMut;
}
/// Rename false type to the underlying message type to further improve the error message.
impl<Msg> IsMessageMutRename for False<Msg> {
    type Value = Msg;
}
/// Can be used to check whether a message is allowed to mutate state.
pub trait IsMessageMut {}
/// Only implemented for messages that are allowed to mutate state.
impl IsMessageMut for MessageIsMut {}
/// Representant for all messages that are allowed to mutate state.
pub struct MessageIsMut {}
