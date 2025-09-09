// Copyright (C) Use Ink (UK) Ltd.
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

//! Traits and interfaces to operate with storage entities.
//!
//! Generally a type is said to be a storage entity if it implements the
//! [`Storable`] trait. This defines certain constants and routines in order
//! to tell a smart contract how to load and store instances of this type
//! from and to the contract's storage.
//!
//! The [`Packed`] shows that the type can be stored into single storage cell.
//! In most cases, collections(`Vec`, `HashMap`, `HashSet` etc.) work only with packed
//! structures.
//!
//! If at least one of the type's fields occupies its own separate storage cell, it is a
//! non-[`Packed`] type because it occupies more than one storage cell.

#![doc(
    html_logo_url = "https://use.ink/img/crate-docs/logo.png",
    html_favicon_url = "https://use.ink/crate-docs/favicon.png"
)]
#![cfg_attr(not(feature = "std"), no_std)]

mod impls;
mod storage;

#[cfg(feature = "std")]
mod layout;

#[cfg(feature = "std")]
pub use self::layout::StorageLayout;
pub use self::{
    impls::{
        AutoKey,
        ManualKey,
        ResolverKey,
    },
    storage::{
        AutoStorableHint,
        Packed,
        Storable,
        StorableHint,
        StorageKey,
        decode_all,
    },
};
