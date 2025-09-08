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

use ink_ir::IsDocAttribute;
use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
};
use quote::{
    format_ident,
    quote,
};
use syn::{
    Attribute,
    Field,
    Fields,
    spanned::Spanned,
};

use super::utils;

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

/// Derives the `ink::metadata::sol::SolErrorMetadata` trait for the given `struct` or
/// `enum`.
pub fn sol_error_metadata_derive(s: synstructure::Structure) -> TokenStream2 {
    match s.ast().data {
        syn::Data::Struct(_) => {
            sol_error_metadata_derive_struct(s)
                .unwrap_or_else(|err| err.to_compile_error())
        }
        syn::Data::Enum(_) => {
            sol_error_metadata_derive_enum(s).unwrap_or_else(|err| err.to_compile_error())
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
    let self_body = utils::body_from_fields(fields, None);

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
    let params_elems = utils::tuple_elems_from_fields(fields, None);

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
                        &#params_elems,
                    ),
                );
                results
            }
        },
    ))
}

/// Derives the `ink::metadata::sol::SolErrorMetadata` trait for the given `struct`.
fn sol_error_metadata_derive_struct(
    s: synstructure::Structure,
) -> syn::Result<TokenStream2> {
    ensure_no_generics(&s, "SolErrorMetadata")?;

    let Some(variant) = s.variants().first() else {
        return Err(syn::Error::new(
            s.ast().span(),
            "can only derive `SolErrorMetadata` for Rust `struct` items",
        ));
    };

    let ident = &s.ast().ident;
    let name = ident.to_string();
    let params = variant.ast().fields.iter().map(param_metadata_from_field);
    let docs = extract_docs(s.ast().attrs.as_slice());
    let metadata_linker = register_metadata(ident);

    Ok(s.bound_impl(
        quote!(::ink::metadata::sol::SolErrorMetadata),
        quote! {
            fn error_specs() -> ::ink::prelude::vec::Vec<::ink::metadata::sol::ErrorMetadata> {
                #metadata_linker

                vec![
                    ::ink::metadata::sol::ErrorMetadata {
                        name: #name.into(),
                        params: vec![ #( #params ),* ],
                        docs: #docs.into(),
                    }
                ]
            }
        },
    ))
}

/// Derives the `ink::sol::SolErrorDecode` trait for the given `enum`.
fn sol_error_decode_derive_enum(s: synstructure::Structure) -> syn::Result<TokenStream2> {
    ensure_no_generics(&s, "SolErrorDecode")?;
    utils::ensure_non_empty_enum(&s, "SolErrorDecode")?;

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
        let variant_body = utils::body_from_fields(fields, None);
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
    utils::ensure_non_empty_enum(&s, "SolErrorEncode")?;

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
                info
                    .ast()
                    .ident
                    .as_ref()
                    .map(|ident| quote!(#ident))
                    .unwrap_or_else(|| {
                        let binding = &info.binding;
                        quote!(#binding)
                    })
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

/// Derives the `ink::metadata::sol::SolErrorMetadata` trait for the given `enum`.
fn sol_error_metadata_derive_enum(
    s: synstructure::Structure,
) -> syn::Result<TokenStream2> {
    ensure_no_generics(&s, "SolErrorMetadata")?;
    utils::ensure_non_empty_enum(&s, "SolErrorMetadata")?;

    let error_variants = s.variants().iter().map(|variant| {
        let variant_ident = variant.ast().ident;
        let variant_name = variant_ident.to_string();
        let params = variant.ast().fields.iter().map(param_metadata_from_field);
        let docs = extract_docs(variant.ast().attrs);

        quote! {
            ::ink::metadata::sol::ErrorMetadata {
                name: #variant_name.into(),
                params: vec![ #( #params ),* ],
                docs: #docs.into(),
            }
        }
    });
    let metadata_linker = register_metadata(&s.ast().ident);

    Ok(s.bound_impl(
        quote!(::ink::metadata::sol::SolErrorMetadata),
        quote! {
            fn error_specs() -> ::ink::prelude::vec::Vec<::ink::metadata::sol::ErrorMetadata> {
                #metadata_linker

                vec![ #( #error_variants ),* ]
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
            format!(
                "can only derive `{trait_name}` for Rust `struct` or `enum` \
                items without generics"
            ),
        ))
    }
}

/// Register an error metadata function in the distributed slice for combining all
/// errors referenced in the contract binary.
fn register_metadata(ident: &Ident) -> TokenStream2 {
    quote! {
       #[::ink::linkme::distributed_slice(::ink::CONTRACT_ERRORS_SOL)]
       #[linkme(crate = ::ink::linkme)]
       static ERROR_METADATA: fn() -> ::ink::prelude::vec::Vec<::ink::metadata::sol::ErrorMetadata> =
           <#ident as ::ink::metadata::sol::SolErrorMetadata>::error_specs;
    }
}

/// Returns the error parameter from the given field.
fn param_metadata_from_field(field: &Field) -> TokenStream2 {
    let ty = &field.ty;
    let name = field
        .ident
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default();
    let docs = extract_docs(field.attrs.as_slice());
    let sol_ty = quote! {
        <#ty as ::ink::SolEncode>::SOL_NAME
    };
    quote! {
        ::ink::metadata::sol::ErrorParamMetadata {
            name: #name.into(),
            ty: #sol_ty.into(),
            docs: #docs.into(),
        }
    }
}

/// Returns the rustdoc string from the given item attributes.
fn extract_docs(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| attr.extract_docs())
        .collect::<Vec<_>>()
        .join("\n")
}
