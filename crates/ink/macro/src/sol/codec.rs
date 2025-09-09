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
    Expr,
    Field,
    Fields,
    GenericParam,
    Lit,
    spanned::Spanned,
};
use synstructure::VariantInfo;

use super::utils;

/// Derives the `ink::SolDecode` trait for the given `struct` or `enum`.
pub fn sol_decode_derive(s: synstructure::Structure) -> TokenStream2 {
    match s.ast().data {
        syn::Data::Struct(_) => {
            sol_decode_derive_struct(s).unwrap_or_else(|err| err.to_compile_error())
        }
        syn::Data::Enum(_) => {
            sol_decode_derive_enum(s).unwrap_or_else(|err| err.to_compile_error())
        }
        _ => {
            syn::Error::new(
                s.ast().span(),
                "can only derive `SolDecode` for Rust `struct` and `enum` items",
            )
            .to_compile_error()
        }
    }
}

/// Derives the `ink::SolEncode` trait for the given `struct` or `enum`.
pub fn sol_encode_derive(s: synstructure::Structure) -> TokenStream2 {
    match s.ast().data {
        syn::Data::Struct(_) => {
            sol_encode_derive_struct(s).unwrap_or_else(|err| err.to_compile_error())
        }
        syn::Data::Enum(_) => {
            sol_encode_derive_enum(s).unwrap_or_else(|err| err.to_compile_error())
        }
        _ => {
            syn::Error::new(
                s.ast().span(),
                "can only derive `SolEncode` for Rust `struct` and `enum` items",
            )
            .to_compile_error()
        }
    }
}

/// Derives the `ink::SolDecode` trait for the given `struct`.
fn sol_decode_derive_struct(s: synstructure::Structure) -> syn::Result<TokenStream2> {
    let Some(variant) = s.variants().first() else {
        return Err(syn::Error::new(
            s.ast().span(),
            "can only derive `SolDecode` for Rust `struct` items",
        ));
    };

    let fields = variant.ast().fields;
    let sol_tys = fields.iter().map(|field| {
        let ty = &field.ty;
        quote! {
            <#ty as ::ink::SolDecode>::SolType
        }
    });
    fn from_sol_type(value: TokenStream2, field: &Field) -> TokenStream2 {
        let ty = &field.ty;
        quote! {
            <#ty as ::ink::SolDecode>::from_sol_type(#value)?
        }
    }
    let self_body = utils::body_from_fields(fields, Some(from_sol_type));

    Ok(s.bound_impl(
        quote!(::ink::SolDecode),
        quote! {
            type SolType = ( #( #sol_tys, )* );

            fn from_sol_type(value: Self::SolType) -> ::core::result::Result<Self, ::ink::sol::Error> {
                Ok(Self #self_body)
            }
        },
    ))
}

/// Derives the `ink::SolEncode` trait for the given `struct`.
fn sol_encode_derive_struct(mut s: synstructure::Structure) -> syn::Result<TokenStream2> {
    let Some(variant) = s.variants().first() else {
        return Err(syn::Error::new(
            s.ast().span(),
            "can only derive `SolEncode` for Rust `struct` items",
        ));
    };

    let fields = variant.ast().fields;
    let sol_tys = fields.iter().map(|field| {
        let ty = &field.ty;
        quote!( <#ty as ::ink::SolEncode<'a>>::SolType )
    });
    fn to_sol_type(value: TokenStream2, field: &Field) -> TokenStream2 {
        let ty = &field.ty;
        quote! {
            <#ty as ::ink::SolEncode<'_>>::to_sol_type(#value)
        }
    }
    let sol_ty_tuple = utils::tuple_elems_from_fields(fields, Some(to_sol_type));

    let lifetime: GenericParam = syn::parse_quote!('a);
    s.add_impl_generic(lifetime);
    Ok(s.bound_impl(
        quote!(::ink::SolEncode<'a>),
        quote! {
            type SolType = ( #( #sol_tys, )* );

            fn to_sol_type(&'a self) -> Self::SolType {
                #sol_ty_tuple
            }
        },
    ))
}

/// Derives the `ink::SolDecode` trait for the given `enum`.
fn sol_decode_derive_enum(s: synstructure::Structure) -> syn::Result<TokenStream2> {
    utils::ensure_non_empty_enum(&s, "SolDecode")?;
    ensure_empty_variants(&s, "SolDecode")?;
    ensure_u8_max_variants(&s, "SolDecode")?;
    ensure_consistent_variant_int_repr(&s, "SolDecode")?;
    ensure_valid_discriminant_values(&s, "SolDecode")?;

    let variants_match = s.variants().iter().enumerate().map(|(idx, variant)| {
        let variant_ident = variant.ast().ident;
        let int_repr = variant_int_repr(variant, idx as u8);
        let field_delimiters = variant_field_delimiters(variant);
        quote! {
            #int_repr => {
                ::core::result::Result::Ok(Self:: #variant_ident #field_delimiters)
            }
        }
    });

    Ok(s.bound_impl(
        quote!(::ink::SolDecode),
        quote! {
            type SolType = ::core::primitive::u8;

            fn from_sol_type(value: Self::SolType) -> ::core::result::Result<Self, ::ink::sol::Error> {
                match value {
                    #( #variants_match )*
                    _ => ::core::result::Result::Err(::ink::sol::Error)
                }
            }
        },
    ))
}

/// Derives the `ink::SolEncode` trait for the given `enum`.
fn sol_encode_derive_enum(mut s: synstructure::Structure) -> syn::Result<TokenStream2> {
    utils::ensure_non_empty_enum(&s, "SolEncode")?;
    ensure_empty_variants(&s, "SolEncode")?;
    ensure_u8_max_variants(&s, "SolEncode")?;
    ensure_consistent_variant_int_repr(&s, "SolEncode")?;
    ensure_valid_discriminant_values(&s, "SolEncode")?;

    let lifetime: GenericParam = syn::parse_quote!('a);
    s.add_impl_generic(lifetime);

    let variants_match = s.variants().iter().enumerate().map(|(idx, variant)| {
        let variant_ident = variant.ast().ident;
        let int_repr = variant_int_repr(variant, idx as u8);
        let field_delimiters = variant_field_delimiters(variant);
        quote! {
            Self:: #variant_ident #field_delimiters => #int_repr,
        }
    });

    Ok(s.bound_impl(
        quote!(::ink::SolEncode<'a>),
        quote! {
            type SolType = ::core::primitive::u8;

            fn to_sol_type(&'a self) -> Self::SolType {
                match self {
                    #( #variants_match )*
                }
            }
        },
    ))
}

/// Ensures that the given item has only unit-only or field-less variant.
fn ensure_empty_variants(
    s: &synstructure::Structure,
    trait_name: &str,
) -> syn::Result<()> {
    let has_non_empty_variants = s
        .variants()
        .iter()
        .any(|variant| !variant.ast().fields.is_empty());
    if has_non_empty_variants {
        Err(syn::Error::new(
            s.ast().span(),
            format!(
                "can only derive `{trait_name}` for Rust `enum` items with \
                only unit-only or field-less variants"
            ),
        ))
    } else {
        Ok(())
    }
}

/// Ensures that the given item has at most `u8::MAX` variants.
///
/// # Note
///
/// Rust doesn't have an explicit limit on the number of allowed enum variants, however,
/// the practical limit can be understood to `isize::MAX` as a `rustc` implementation
/// detail.
///
/// References:
///
/// - <https://doc.rust-lang.org/reference/items/enumerations.html#r-items.enum.discriminant.repr-rust>
/// - <https://github.com/rust-lang/rust/blob/f63685ddf3d3c92a61158cd55d44bde17c2b024f/compiler/rustc_ast/src/ast.rs#L3270>
fn ensure_u8_max_variants(
    s: &synstructure::Structure,
    trait_name: &str,
) -> syn::Result<()> {
    if s.variants().len() > u8::MAX as usize {
        Err(syn::Error::new(
            s.ast().span(),
            format!(
                "can only derive `{trait_name}` for Rust `enum` items \
                with at most `u8::MAX` variants"
            ),
        ))
    } else {
        Ok(())
    }
}

/// Ensures that the given item will yield a consistent integer representation for all its
/// variants.
///
/// # Note
///
/// This check only succeeds if one of the following conditions is met:
/// - No variant has an explicitly set discriminant
/// - All variants have an explicitly set discriminant
fn ensure_consistent_variant_int_repr(
    s: &synstructure::Structure,
    trait_name: &str,
) -> syn::Result<()> {
    let n_variants_with_discriminants = s
        .variants()
        .iter()
        .filter(|variant| variant.ast().discriminant.is_some())
        .count();
    if n_variants_with_discriminants > 0
        && n_variants_with_discriminants != s.variants().len()
    {
        Err(syn::Error::new(
            s.ast().span(),
            format!(
                "can only derive `{trait_name}` for Rust `enum` items that \
                either have no variants with explicitly specified discriminants, \
                or have explicitly specified discriminants for all variants"
            ),
        ))
    } else {
        Ok(())
    }
}

/// Ensures that the given item has only valid discriminant values, if any are explicitly
/// specified.
///
/// # Note
///
/// Enums are encoded as `u8` (i.e. `uint8` in Solidity ABI encoding), so the maximum
/// allowed discriminant value is `u8::MAX` (i.e. `255`)
///
/// Rust does NOT have an explicit limit on the number of allowed enum variants, however,
/// the practical limit can be understood to `isize::MAX` as a `rustc` implementation
/// detail.
///
/// For unit-only enums, `rustc` **currently** enforces that any explicitly specified enum
/// discriminant is not larger than `u8::MAX` if (and only if) the number of variants is
/// also less than `u8::MAX`.
///
/// References:
///
/// - <https://doc.rust-lang.org/reference/items/enumerations.html#r-items.enum.discriminant.repr-rust>
/// - <https://github.com/rust-lang/rust/blob/f63685ddf3d3c92a61158cd55d44bde17c2b024f/compiler/rustc_ast/src/ast.rs#L3270>
fn ensure_valid_discriminant_values(
    s: &synstructure::Structure,
    trait_name: &str,
) -> syn::Result<()> {
    let offending_span = s.variants().iter().find_map(|variant| {
        variant.ast().discriminant.as_ref().and_then(|(_, expr)| {
            match expr {
                Expr::Lit(expr) => {
                    match &expr.lit {
                        Lit::Int(value) => {
                            let discr = value.base10_parse::<usize>().ok()?;
                            (discr > u8::MAX as usize).then_some(expr.span())
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        })
    });
    if let Some(span) = offending_span {
        Err(syn::Error::new(
            span,
            format!(
                "can only derive `{trait_name}` for Rust `enum` items \
                with discriminant values (if explicitly specified) \
                not larger than `u8::MAX`"
            ),
        ))
    } else {
        Ok(())
    }
}

/// Returns an integer representation of an enum variant.
fn variant_int_repr(variant: &VariantInfo, idx: u8) -> TokenStream2 {
    variant
        .ast()
        .discriminant
        .as_ref()
        .map(|(_, expr)| quote!( #expr ))
        .unwrap_or_else(|| quote! ( #idx ))
}

/// Returns the field delimiters for given a variant.
fn variant_field_delimiters(variant: &VariantInfo) -> TokenStream2 {
    match variant.ast().fields {
        Fields::Named(_) => quote!({}),
        Fields::Unnamed(_) => quote! { () },
        Fields::Unit => quote!(),
    }
}
