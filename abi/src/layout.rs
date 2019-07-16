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

use type_metadata::{
	// Metadata,
	form::{
		Form,
		MetaForm,
	},
};
use derive_more::From;
use serde::{Serialize, Serializer};

// impl<T> HasLayout for ink_core::storage::Value<T>
// where
//     T: Metadata,
// {
// 	fn layout(&self) -> LayoutStruct {
// 		LayoutStruct {
// 			fields: vec![
// 				LayoutField {
// 					name: "value",
// 					ty: T::meta_type(),
// 					kind: KeyRange {
// 						key: Key::from([0xDEAD_BEEF; 32]),
// 						len: 1,
// 					}.into()
// 				}
// 			],
// 		}
// 	}
// }

// impl<T> HasLayout for ink_core::storage::Vec<T>
// where
//     T: Metadata,
// {
// 	fn layout(&self) -> LayoutStruct {
// 		LayoutStruct {
// 			fields: vec![
// 				LayoutField {
// 					name: "len",
// 					ty: u32::meta_type(),
// 					kind: KeyRange {
// 						key: Key::from([0xBEEF_DEAD; 32]),
// 						len: 1,
// 					}.into()
// 				},
// 				LayoutField {
// 					name: "data",
// 					ty: T::meta_type(),
// 					kind: KeyRange {
// 						key: Key::from([0xBEEF_BEEF; 32]),
// 						len: 4294967296,
// 					}.into()
// 				}
// 			],
// 		}
// 	}
// }

/// A concrete range of keys.
#[derive(Debug, PartialEq, Eq, From)]
pub struct Key(ink_core::storage::Key);

impl Serialize for Key {
	fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		unimplemented!()
	}
}

impl From<[u8; 32]> for Key {
	fn from(array: [u8; 32]) -> Self {
		Key(ink_core::storage::Key(array))
	}
}

/// A concrete range of keys.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct KeyRange {
    /// The offset key.
    key: Key,
    /// The number of keys that are included in the layout bound
    /// starting from the offset key.
    ///
    /// Note that for simplicity `len` normally is either 1 (cell) or 2^32 (chunk).
    len: u32,
}

/// Implemented by types that have a storage layout.
pub trait HasLayout {
    /// Returns teh storage layout of `self`.
    fn layout(&self) -> LayoutStruct;
}

/// A struct storage layout.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound =	"F::TypeId: Serialize")]
pub struct LayoutStruct<F: Form = MetaForm> {
    /// The sub-fields of the struct.
    fields: Vec<LayoutField<F>>,
}

/// The layout for a particular field of a struct layout.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound =	"F::TypeId: Serialize")]
pub struct LayoutField<F: Form = MetaForm> {
    /// The name of the field.
    name: F::String,
    /// The type identifier of the field.
    #[serde(rename = "type")]
    ty: F::TypeId,
    /// The kind of the field.
    ///
    /// This is either a direct layout bound
    /// or another recursive layout sub-struct.
    kind: LayoutKind<F>,
}

/// Either a concrete layout bound or another layout sub-struct.
#[derive(Debug, PartialEq, Eq, Serialize, From)]
#[serde(bound =	"F::TypeId: Serialize")]
pub enum LayoutKind<F: Form = MetaForm> {
    /// A concrete layout bound.
    Range(KeyRange),
    /// A nested sub-struct with layout bounds.
    Fields(LayoutStruct<F>),
}
