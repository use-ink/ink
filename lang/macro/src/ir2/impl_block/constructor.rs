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

use crate::{
    error::ExtError as _,
    ir2,
    ir2::attrs::Attrs as _,
};
use core::convert::TryFrom;
use syn::spanned::Spanned as _;

/// An ink! constructor definition.
#[derive(Debug, PartialEq, Eq)]
pub struct Constructor {
    /// The underlying Rust method item.
    item: syn::ImplItemMethod,
    /// If the ink! constructor can receive funds.
    is_payable: bool,
    /// An optional user provided salt.
    salt: Option<ir2::Salt>,
    /// An optional user provided selector.
    selector: Option<ir2::Selector>,
}

impl TryFrom<syn::ImplItemMethod> for Constructor {
    type Error = syn::Error;

    fn try_from(method_item: syn::ImplItemMethod) -> Result<Self, Self::Error> {
        let method_span = method_item.span();
        let (ink_attrs, other_attrs) = ir2::sanitize_attributes(
            method_span,
            method_item.attrs,
            &ir2::AttributeArgKind::Constructor,
            |kind| {
                match kind {
                    ir2::AttributeArgKind::Constructor |
                    ir2::AttributeArgKind::Payable |
                    ir2::AttributeArgKind::Salt(_) |
                    ir2::AttributeArgKind::Selector(_) => false,
                    _ => false,
                }
            }
        )?;
        let is_payable = false; // TODO
        let salt = None; // TODO
        let selector = None; // TODO
        Ok(Constructor {
            is_payable,
            salt,
            selector,
            item: syn::ImplItemMethod {
                attrs: other_attrs,
                ..
                method_item
            },
        })
    }
}
