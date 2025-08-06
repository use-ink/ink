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

use ink_ir::{
    EventConfig,
    IsDocAttribute,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::spanned::Spanned;

/// Derives the `ink::Event` trait for the given `struct`.
pub fn event_metadata_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::Move)
        .add_bounds(synstructure::AddBounds::Fields)
        .underscore_const(true);
    match &s.ast().data {
        syn::Data::Struct(_) => {
            event_metadata_derive_struct(s).unwrap_or_else(|err| err.to_compile_error())
        }
        _ => {
            syn::Error::new(
                s.ast().span(),
                "can only derive `EventMetadata` for Rust `struct` items",
            )
            .to_compile_error()
        }
    }
}

/// `Event` derive implementation for `struct` types.
fn event_metadata_derive_struct(s: synstructure::Structure) -> syn::Result<TokenStream2> {
    assert_eq!(s.variants().len(), 1, "can only operate on structs");
    let span = s.ast().span();

    let variant = &s.variants()[0];
    let ident = variant.ast().ident;

    let config = EventConfig::try_from(variant.ast().attrs)?;
    let name = config
        .name()
        .map(ToString::to_string)
        .unwrap_or_else(|| variant.ast().ident.to_string());

    let docs = variant
        .ast()
        .attrs
        .iter()
        .filter_map(|attr| attr.extract_docs());

    let args = variant.bindings().iter().map( |field| {
        let field_ty = &field.ast().ty;
        let field_span = field_ty.span();
        if let Some(field_name) = field.ast().ident.as_ref() {
            let indexed = super::has_ink_topic_attribute(field)?;
            let docs = field
                .ast()
                .attrs
                .iter()
                .filter_map(|attr| attr.extract_docs());
            let ty_spec = ink_codegen::generate_type_spec(field_ty);
            Ok(quote_spanned!(field_span =>
                ::ink::metadata::EventParamSpec::new(::core::stringify!(#field_name))
                    .of_type(#ty_spec)
                    .indexed(#indexed)
                    .docs([ #( #docs ),* ])
                    .done()
            ))
        } else {
            Err(syn::Error::new(
                field_span,
                "can only derive `EventMetadata` for Rust `struct` items with named fields",
            ))
        }
    }).collect::<syn::Result<Vec<_>>>()?;

    Ok(s.bound_impl(
        quote_spanned!(span=> ::ink::metadata::EventMetadata),
        quote_spanned!(span=>
            const MODULE_PATH: &'static str = ::core::module_path!();

            fn event_spec() -> ::ink::metadata::EventSpec {
               // register this event metadata function in the distributed slice for combining all
               // events referenced in the contract binary.
               #[::ink::linkme::distributed_slice(::ink::CONTRACT_EVENTS)]
               #[linkme(crate = ::ink::linkme)]
               static EVENT_METADATA: fn() -> ::ink::metadata::EventSpec =
                   <#ident as ::ink::metadata::EventMetadata>::event_spec;

                ::ink::metadata::EventSpec::new(#name)
                    .module_path(::core::module_path!())
                    .signature_topic(
                        <Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC
                    )
                    .args([
                       #( #args ),*
                    ])
                    .docs([
                       #( #docs ),*
                    ])
                    .done()
            }
        ),
    ))
}
