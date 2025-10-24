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

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Field,
    Fields,
    spanned::Spanned,
};

/// Ensures that the given item has at least one variant.
pub fn ensure_non_empty_enum(
    s: &synstructure::Structure,
    trait_name: &str,
) -> syn::Result<()> {
    if s.variants().is_empty() {
        Err(syn::Error::new(
            s.ast().span(),
            format!(
                "can only derive `{trait_name}` for Rust `enum` items \
                with at least one variant"
            ),
        ))
    } else {
        Ok(())
    }
}

/// Composes the body for the variant or struct given its fields.
pub fn body_from_fields(
    fields: &Fields,
    transformer: Option<fn(TokenStream2, &Field) -> TokenStream2>,
) -> TokenStream2 {
    let from_params_elems = || {
        fields.iter().enumerate().map(|(idx, field)| {
            let idx = syn::Index::from(idx);
            let value = match &transformer {
                None => quote!(value.#idx),
                Some(transformer) => transformer(quote!(value.#idx), field),
            };
            match &field.ident {
                // Handles named fields.
                None => quote!(#value),
                // Handles tuple elements.
                Some(ident) => {
                    quote! {
                        #ident: #value
                    }
                }
            }
        })
    };
    match fields {
        // Handles named fields.
        Fields::Named(_) => {
            let self_fields = from_params_elems();
            quote!(
                {
                    #( #self_fields, )*
                }
            )
        }
        // Handles tuple elements.
        Fields::Unnamed(_) => {
            let self_elems = from_params_elems();
            quote! {
                ( #( #self_elems, )* )
            }
        }
        // Handles unit variants.
        Fields::Unit => quote!(),
    }
}

/// Composes a list of tuple elements for the variant or struct given its fields.
pub fn tuple_elems_from_fields(
    fields: &Fields,
    transformer: Option<fn(TokenStream2, &Field) -> TokenStream2>,
) -> TokenStream2 {
    let elems = fields.iter().enumerate().map(|(idx, field)| {
        // Accessor is either a field name or tuple index.
        let accessor = field
            .ident
            .as_ref()
            .map(|ident| quote!(#ident))
            .unwrap_or_else(|| {
                let idx = syn::Index::from(idx);
                quote!(#idx)
            });
        match &transformer {
            None => quote!(&self.#accessor),
            Some(transformer) => transformer(quote!(&self.#accessor), field),
        }
    });
    quote! {
        ( #( #elems, )* )
    }
}
