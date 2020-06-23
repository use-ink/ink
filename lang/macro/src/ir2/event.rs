// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use crate::ir2;
use core::convert::TryFrom;

pub struct Event {
    ast: syn::ItemStruct,
}

impl TryFrom<syn::ItemStruct> for Event {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self, Self::Error> {
        todo!()
impl Event {
    /// Returns the identifier of the event struct.
    pub fn ident(&self) -> &Ident {
        &self.ast.ident
    }

    /// Returns an iterator yielding all fields of the event struct.
    pub fn fields(&self) -> syn::punctuated::Iter<syn::Field> {
        self.ast.fields.iter()
    }

    /// Returns an iterator yielding all the `#[ink(topic)]` annotated fields
    /// of the event struct.
    pub fn topic_fields(&self) -> TopicFieldsIter {
        TopicFieldsIter::new(self)
    }
}

/// Iterator yielding all `#[ink(topic)]` annotated fields of an event struct.
pub struct TopicFieldsIter<'a> {
    iter: syn::punctuated::Iter<'a, syn::Field>,
}

impl<'a> TopicFieldsIter<'a> {
    /// Creates a new topics fields iterator for the given ink! event struct.
    fn new(event: &'a Event) -> Self {
        Self {
            iter: event.fields(),
        }
    }
}

impl<'a> Iterator for TopicFieldsIter<'a> {
    type Item = &'a syn::Field;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.iter.next() {
                None => return None,
                Some(field) => {
                    if ir2::first_ink_attribute(&field.attrs)
                        .unwrap_or_default()
                        .map(|field| {
                            field.first().kind() == &ir2::AttributeArgKind::Event
                        })
                        .unwrap_or(false)
                    {
                        return Some(field)
                    }
                    continue 'outer
                }
            }
        }
    }
}
    }
}
