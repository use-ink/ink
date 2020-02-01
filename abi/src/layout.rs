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

#[cfg(not(feature = "std"))]
use alloc::{
    string::String,
    vec::Vec,
};
use core::fmt::Write;

use derive_more::From;
use serde::{
    Serialize,
    Serializer,
};
use type_metadata::{
    form::{
        CompactForm,
        Form,
        MetaForm,
    },
    IntoCompact,
    Registry,
};

/// Implemented by types that have a storage layout.
///
/// Has to be used on previously allocated instances of the types.
pub trait HasLayout {
    fn layout(&self) -> StorageLayout;
}

impl From<ink_primitives::Key> for LayoutKey {
    fn from(key: ink_primitives::Key) -> Self {
        LayoutKey(key.0)
    }
}

impl HasLayout for ink_primitives::Key {
    fn layout(&self) -> StorageLayout {
        LayoutRange::cell(*self, <[u8; 32] as type_metadata::Metadata>::meta_type())
            .into()
    }
}

/// Either a concrete layout bound or another layout sub-struct.
#[derive(Debug, PartialEq, Eq, Serialize, From)]
#[serde(bound = "F::TypeId: Serialize")]
#[serde(untagged)]
pub enum StorageLayout<F: Form = MetaForm> {
    /// A concrete layout bound.
    Range(LayoutRange<F>),
    /// A nested sub-struct with layout bounds.
    Struct(LayoutStruct<F>),
}

impl IntoCompact for StorageLayout {
    type Output = StorageLayout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        match self {
            StorageLayout::Range(layout_range) => {
                StorageLayout::Range(layout_range.into_compact(registry))
            }
            StorageLayout::Struct(layout_struct) => {
                StorageLayout::Struct(layout_struct.into_compact(registry))
            }
        }
    }
}

/// A concrete range of keys.
///
/// Basically a thin-wrapper around keys from `ink_core` library for serialization purposes.
#[derive(Debug, PartialEq, Eq, From, Serialize)]
#[serde(transparent)]
pub struct LayoutKey(
    /// Internals must be compatible with `ink_primitives::Key`.
    pub [u8; 32],
);

/// A struct storage layout.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct LayoutStruct<F: Form = MetaForm> {
    #[serde(rename = "struct.type")]
    self_ty: F::TypeId,
    /// The sub-fields of the struct.
    #[serde(rename = "struct.fields")]
    fields: Vec<LayoutField<F>>,
}

impl LayoutStruct {
    /// Creates a new layout struct.
    pub fn new<F>(self_ty: <MetaForm as Form>::TypeId, fields: F) -> Self
    where
        F: IntoIterator<Item = LayoutField>,
    {
        Self {
            self_ty,
            fields: fields.into_iter().collect::<Vec<_>>(),
        }
    }
}

impl IntoCompact for LayoutStruct {
    type Output = LayoutStruct<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        LayoutStruct {
            self_ty: registry.register_type(&self.self_ty),
            fields: self
                .fields
                .into_iter()
                .map(|field| field.into_compact(registry))
                .collect::<Vec<_>>(),
        }
    }
}

/// The layout for a particular field of a struct layout.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct LayoutField<F: Form = MetaForm> {
    /// The name of the field.
    name: F::String,
    /// The kind of the field.
    ///
    /// This is either a direct layout bound
    /// or another recursive layout sub-struct.
    #[serde(rename = "layout")]
    sub_layout: StorageLayout<F>,
}

impl LayoutField {
    /// Creates a new layout field from the given name and sub-structural layout.
    pub fn new(name: <MetaForm as Form>::String, sub_layout: StorageLayout) -> Self {
        Self { name, sub_layout }
    }

    /// Creates a new layout field for the given field instance.
    pub fn of<T>(name: <MetaForm as Form>::String, field: &T) -> Self
    where
        T: HasLayout,
    {
        Self::new(name, field.layout())
    }
}

impl IntoCompact for LayoutField {
    type Output = LayoutField<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        LayoutField {
            name: registry.register_string(self.name),
            sub_layout: self.sub_layout.into_compact(registry),
        }
    }
}

/// Direct range of associated storage keys.
///
/// This may represent either a single cell, a whole chunk of cells or something in between.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct LayoutRange<F: Form = MetaForm> {
    /// The single key for cells or the starting key address for chunks.
    #[serde(rename = "range.offset", serialize_with = "serialize_key")]
    offset: LayoutKey,
    /// The amount of associated key addresses starting from the offset key.
    #[serde(rename = "range.len")]
    len: u32,
    /// The element type stored under the associated keys.
    #[serde(rename = "range.elem_type")]
    elem_ty: F::TypeId,
}

impl IntoCompact for LayoutRange {
    type Output = LayoutRange<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        LayoutRange {
            offset: self.offset,
            len: self.len,
            elem_ty: registry.register_type(&self.elem_ty),
        }
    }
}

impl LayoutRange {
    /// Creates a layout range representing a single cell.
    pub fn cell<K>(at: K, elem_ty: <MetaForm as Form>::TypeId) -> Self
    where
        K: Into<LayoutKey>,
    {
        Self {
            offset: at.into(),
            len: 1,
            elem_ty,
        }
    }

    /// Creates a layout range for a whole chunk starting at the offset key.
    pub fn chunk<K>(offset: K, elem_ty: <MetaForm as Form>::TypeId) -> Self
    where
        K: Into<LayoutKey>,
    {
        Self {
            offset: offset.into(),
            len: 0xFFFF_FFFF,
            elem_ty,
        }
    }
}

fn serialize_key<S>(key: &LayoutKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let bytes = key.0;
    let mut hex = String::with_capacity(bytes.len() * 2 + 2);
    write!(hex, "0x").expect("failed writing to string");
    for byte in &bytes {
        write!(hex, "{:02x}", byte).expect("failed writing to string");
    }

    serializer.serialize_str(&hex)
}

#[cfg(test)]
mod tests {
    use super::*;
    use type_metadata::{
        form::{
            Form,
            MetaForm,
        },
        IntoCompact,
        Registry,
    };

    #[test]
    fn key_must_serialize_to_hex() {
        // given
        let type_id = <MetaForm as Form>::TypeId::new::<u32>();
        let offset = LayoutKey([1; 32]);
        let cs: LayoutRange<MetaForm> = LayoutRange {
            offset,
            len: 1337,
            elem_ty: type_id,
        };
        let mut registry = Registry::new();

        // when
        let json = serde_json::to_string(&cs.into_compact(&mut registry)).unwrap();

        // then
        assert_eq!(
            json,
            "{\"range.offset\":\"0x0101010101010101010101010101010101010101010101010101010101010101\",\"range.len\":1337,\"range.elem_type\":1}"
        );
    }
}
