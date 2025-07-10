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
use quote::{
    format_ident,
    quote,
};
use syn::{
    spanned::Spanned,
    Fields,
};

/// Derives the `ink::sol::SolErrorDecode` trait for the given `struct` or `enum`.
pub fn sol_error_decode_derive(s: synstructure::Structure) -> TokenStream2 {
    match s.ast().data {
        syn::Data::Struct(_) => {
            sol_error_decode_derive_struct(s).unwrap_or_else(|err| err.to_compile_error())
        }
        syn::Data::Enum(_) => {
            sol_error_decode_derive_enum(s).unwrap_or_else(|err| err.to_compile_error())
        }
        _ => {
            syn::Error::new(
                s.ast().span(),
                "can only derive `SolErrorDecode` for Rust `struct` and `enum` items",
            )
            .to_compile_error()
        }
    }
}

/// Derives the `ink::sol::SolErrorEncode` trait for the given `struct` or `enum`.
pub fn sol_error_encode_derive(s: synstructure::Structure) -> TokenStream2 {
    match s.ast().data {
        syn::Data::Struct(_) => {
            sol_error_encode_derive_struct(s).unwrap_or_else(|err| err.to_compile_error())
        }
        syn::Data::Enum(_) => {
            sol_error_encode_derive_enum(s).unwrap_or_else(|err| err.to_compile_error())
        }
        _ => {
            syn::Error::new(
                s.ast().span(),
                "can only derive `SolErrorEncode` for Rust `struct` and `enum` items",
            )
            .to_compile_error()
        }
    }
}

/// Derives the `ink::sol::SolErrorDecode` trait for the given `struct`.
fn sol_error_decode_derive_struct(
    s: synstructure::Structure,
) -> syn::Result<TokenStream2> {
    ensure_no_generics(&s, "SolErrorDecode")?;

    let Some(variant) = s.variants().first() else {
        return Err(syn::Error::new(
            s.ast().span(),
            "can only derive `SolErrorDecode` for Rust `struct` items",
        ));
    };

    let name = &s.ast().ident.to_string();
    let fields = variant.ast().fields;
    let params_tys = fields.iter().map(|field| &field.ty);
    let params_tuple_ty = quote! {
        ( #( #params_tys, )* )
    };
    let self_body = body_from_fields(fields);

    Ok(s.bound_impl(
        quote!(::ink::sol::SolErrorDecode),
        quote! {
            fn decode(data: &[::core::primitive::u8]) -> ::core::result::Result<Self, ::ink::sol::Error>
            where
                Self: Sized,
            {
                const SELECTOR: [::core::primitive::u8; 4] = ::ink::sol_error_selector!(#name, #params_tuple_ty);
                if data[..4] == SELECTOR {
                    <#params_tuple_ty as ::ink::sol::SolParamsDecode>::decode(
                        &data[4..],
                    )
                    .map(|value| {
                        Self #self_body
                    })
                } else {
                    Err(::ink::sol::Error)
                }
            }
        },
    ))
}

/// Derives the `ink::sol::SolErrorEncode` trait for the given `struct`.
fn sol_error_encode_derive_struct(
    s: synstructure::Structure,
) -> syn::Result<TokenStream2> {
    ensure_no_generics(&s, "SolErrorEncode")?;

    let Some(variant) = s.variants().first() else {
        return Err(syn::Error::new(
            s.ast().span(),
            "can only derive `SolErrorEncode` for Rust `struct` items",
        ));
    };

    let name = &s.ast().ident.to_string();
    let fields = variant.ast().fields;
    let selector_params_tys = fields.iter().map(|field| &field.ty);
    let encode_params_tys = fields.iter().map(|field| {
        let ty = &field.ty;
        quote!( &#ty )
    });
    let params_elems = fields.iter().enumerate().map(|(idx, field)| {
        // Accessor is either a field name or tuple index.
        let accessor = field
            .ident
            .as_ref()
            .map(|ident| quote!(#ident))
            .unwrap_or_else(|| {
                let idx = syn::Index::from(idx);
                quote!(#idx)
            });
        quote!( &self.#accessor )
    });

    Ok(s.bound_impl(
        quote!(::ink::sol::SolErrorEncode),
        quote! {
            fn encode(&self) -> ::ink::prelude::vec::Vec<::core::primitive::u8> {
                let mut results = ::ink::prelude::vec::Vec::from(
                    ::ink::sol_error_selector!(
                        #name,
                        ( #( #selector_params_tys, )* )
                    )
                );
                results.extend(
                    <( #( #encode_params_tys, )* ) as ::ink::sol::SolParamsEncode>::encode(
                        &( #( #params_elems, )* ),
                    ),
                );
                results
            }
        },
    ))
}

/// Derives the `ink::sol::SolErrorDecode` trait for the given `enum`.
fn sol_error_decode_derive_enum(s: synstructure::Structure) -> syn::Result<TokenStream2> {
    ensure_no_generics(&s, "SolErrorDecode")?;
    ensure_non_empty_enum(&s, "SolErrorDecode")?;

    let variant_selector_ident = |idx: usize| format_ident!("VARIANT_{}", idx);
    let variant_selectors = s.variants().iter().enumerate().map(|(idx, variant)| {
        let selector_ident = variant_selector_ident(idx);
        let variant_name = variant.ast().ident.to_string();
        let fields = variant.ast().fields;
        let param_tys = fields.iter().map(|field| &field.ty);
        quote! {
            const #selector_ident: [::core::primitive::u8; 4] = ::ink::sol_error_selector!(
                #variant_name, ( #( #param_tys, )* )
            );
        }
    });
    let variants_match = s.variants().iter().enumerate().map(|(idx, variant)| {
        let variant_ident = variant.ast().ident;
        let selector_ident = variant_selector_ident(idx);
        let fields = variant.ast().fields;
        let param_tys = fields.iter().map(|field| &field.ty);
        let variant_body = body_from_fields(fields);
        quote! {
            #selector_ident => {
                <( #( #param_tys, )* ) as ::ink::sol::SolParamsDecode>::decode(
                    &data[4..],
                )
                .map(|value| {
                    Self:: #variant_ident #variant_body
                })
            }
        }
    });

    Ok(s.bound_impl(
        quote!(::ink::sol::SolErrorDecode),
        quote! {
            fn decode(data: &[::core::primitive::u8]) -> ::core::result::Result<Self, ::ink::sol::Error>
            where
                Self: Sized,
            {
                let selector: [::core::primitive::u8; 4] = data[..4].try_into().map_err(|_| ::ink::sol::Error)?;

                #( #variant_selectors )*

                match selector {
                    #( #variants_match )*
                    _ => Err(::ink::sol::Error),
                }
            }
        },
    ))
}

/// Derives the `ink::sol::SolErrorEncode` trait for the given `enum`.
fn sol_error_encode_derive_enum(s: synstructure::Structure) -> syn::Result<TokenStream2> {
    ensure_no_generics(&s, "SolErrorEncode")?;
    ensure_non_empty_enum(&s, "SolErrorEncode")?;

    let variants_match = s.variants().iter().map(|variant| {
        let variant_ident = variant.ast().ident;
        let variant_name = variant_ident.to_string();
        let fields = variant.ast().fields;
        let selector_params_tys = fields.iter().map(|field| &field.ty);
        let encode_params_tys = fields.iter().map(|field| {
            let ty = &field.ty;
            quote!( &#ty )
        });
        let bindings = || {
            variant.bindings().iter().map(|info| {
                // var is either a field name, or generated "binding_*" name for tuple
                // elements.
                let var_name = info
                    .ast()
                    .ident
                    .as_ref()
                    .map(|ident| quote!(#ident))
                    .unwrap_or_else(|| {
                        let binding = &info.binding;
                        quote!(#binding)
                    });
                var_name
            })
        };
        let (variant_bindings, params_elems) = match fields {
            // Handles named fields.
            Fields::Named(_) => {
                let variant_fields = bindings();
                let params_elems = quote! {
                    #( #variant_fields, )*
                };
                (
                    quote!(
                        {
                            #params_elems
                        }
                    ),
                    params_elems,
                )
            }
            // Handles tuple elements.
            Fields::Unnamed(_) => {
                let variant_elems = bindings();
                let params_elems = quote! {
                    #( #variant_elems, )*
                };
                (
                    quote! {
                        ( #params_elems )
                    },
                    params_elems,
                )
            }
            // Handles unit variants.
            Fields::Unit => (quote!(), quote!()),
        };

        quote! {
            Self:: #variant_ident #variant_bindings => {
                let mut results = ::ink::prelude::vec::Vec::from(
                    ::ink::sol_error_selector!(
                        #variant_name,
                        ( #( #selector_params_tys, )* )
                    )
                );
                results.extend(
                    <( #( #encode_params_tys, )* ) as ::ink::sol::SolParamsEncode>::encode(
                        &( #params_elems ),
                    ),
                );
                results
            }
        }
    });

    Ok(s.bound_impl(
        quote!(::ink::sol::SolErrorEncode),
        quote! {
            fn encode(&self) -> ::ink::prelude::vec::Vec<::core::primitive::u8> {
                match self {
                    #( #variants_match )*
                }
            }
        },
    ))
}

/// Ensures that the given item has no generics.
fn ensure_no_generics(s: &synstructure::Structure, trait_name: &str) -> syn::Result<()> {
    if s.ast().generics.params.is_empty() {
        Ok(())
    } else {
        Err(syn::Error::new(
            s.ast().generics.params.span(),
            format!("can only derive `{trait_name}` for Rust `struct` or `enum` items without generics"),
        ))
    }
}

/// Ensures that the given item has at least one variant.
fn ensure_non_empty_enum(
    s: &synstructure::Structure,
    trait_name: &str,
) -> syn::Result<()> {
    if s.variants().is_empty() {
        Err(syn::Error::new(
            s.ast().span(),
            format!("can only derive `{trait_name}` for Rust `enum` items with at least one variant"),
        ))
    } else {
        Ok(())
    }
}

/// Composes the body for the variant or struct given its fields.
fn body_from_fields(fields: &Fields) -> TokenStream2 {
    let from_params_elems = || {
        fields.iter().enumerate().map(|(idx, field)| {
            let idx = syn::Index::from(idx);
            match &field.ident {
                // Handles named fields.
                None => quote!(value.#idx),
                // Handles tuple elements.
                Some(ident) => {
                    quote! {
                        #ident: value.#idx
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
