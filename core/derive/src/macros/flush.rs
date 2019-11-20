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

use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
};
use syn::{
    self,
    parse::Result,
    DeriveInput,
    Token,
};

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
}

fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
    let input: DeriveInput = syn::parse2(input)?;
    match input.data {
        syn::Data::Struct(data_struct) => {
            generate_for_struct(syn::ItemStruct {
                attrs: input.attrs,
                vis: input.vis,
                struct_token: data_struct.struct_token,
                ident: input.ident,
                generics: input.generics,
                fields: data_struct.fields,
                semi_token: data_struct.semi_token,
            })
        }
        syn::Data::Enum(data_enum) => {
            generate_for_enum(syn::ItemEnum {
                attrs: input.attrs,
                vis: input.vis,
                enum_token: data_enum.enum_token,
                ident: input.ident,
                generics: input.generics,
                variants: data_enum.variants,
                brace_token: data_enum.brace_token,
            })
        }
        syn::Data::Union(data_union) => {
            bail!(data_union.union_token, "cannot derive `Flush` for unions")
        }
    }
}

fn generate_for_struct(mut input: syn::ItemStruct) -> Result<TokenStream2> {
    let mut where_clause = input.generics.make_where_clause().clone();
    for ty in input.fields.iter().map(|field| &field.ty) {
        where_clause
            .predicates
            .push(syn::WherePredicate::Type(syn::PredicateType {
                lifetimes: None,
                bounded_ty: ty.clone(),
                colon_token: <Token![:]>::default(),
                bounds: {
                    let mut bounds = syn::punctuated::Punctuated::new();
                    bounds.push(syn::parse_quote! { ink_core::storage::Flush });
                    bounds
                },
            }))
    }
    let (impl_generics, type_generics, _) = input.generics.split_for_impl();
    let ident = &input.ident;
    let flush_impl = input.fields.iter().enumerate().map(|(n, field)| {
        let ident_or_id = field
            .ident
            .clone()
            .map(|ident| quote! { #ident })
            .unwrap_or(quote! { #n });
        quote! {
            ink_core::storage::Flush::flush(&mut self.#ident_or_id);
        }
    });
    Ok(quote! {
        const _: () = {
            impl #impl_generics ink_core::storage::Flush for #ident #type_generics #where_clause {
                fn flush(&mut self) {
                    #( #flush_impl )*
                }
            }
        };
    })
}

fn generate_for_enum(mut input: syn::ItemEnum) -> Result<TokenStream2> {
    // Returns early if we have an empty set of variants.
    if input.variants.is_empty() {
        return Ok(quote! {})
    }

    // For all field types of all variants in the enum add a proper
    // trait bound to the outer where clause.
    let mut where_clause = input.generics.make_where_clause().clone();
    for ty in input
        .variants
        .iter()
        .map(|variant| variant.fields.iter())
        .flatten()
        .map(|field| &field.ty)
    {
        where_clause
            .predicates
            .push(syn::WherePredicate::Type(syn::PredicateType {
                lifetimes: None,
                bounded_ty: ty.clone(),
                colon_token: <Token![:]>::default(),
                bounds: {
                    let mut bounds = syn::punctuated::Punctuated::new();
                    bounds.push(syn::parse_quote! { ink_core::storage::Flush });
                    bounds
                },
            }))
    }

    let (impl_generics, type_generics, _) = input.generics.split_for_impl();
    let ident = &input.ident;
    let flush_impl = input.variants.iter().map(|variant| {
        let ident = &variant.ident;
        match &variant.fields {
            syn::Fields::Unit => {
                quote! {
                    Self::#ident => (),
                }
            }
            syn::Fields::Named(named_fields) => {
                let field_idents = named_fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap());
                let flush_impl = named_fields.named.iter().map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    quote! {
                        ink_core::storage::Flush::flush(#ident);
                    }
                });
                quote! {
                    Self::#ident { #(#field_idents),* } => {
                        #(
                            #flush_impl
                        )*
                    }
                }
            }
            syn::Fields::Unnamed(unnamed_fields) => {
                let idents_and_impls = unnamed_fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(n, _)| format_ident!("_{}", n))
                    .map(|ident| {
                        (
                            ident.clone(),
                            quote! {
                                ink_core::storage::Flush::flush(#ident);
                            },
                        )
                    });
                let field_idents =
                    idents_and_impls.clone().map(|(ident, _flush_impl)| ident);
                let flush_impl = idents_and_impls
                    .clone()
                    .map(|(_ident, flush_impl)| flush_impl);
                quote! {
                    Self::#ident( #(#field_idents),* ) => {
                        #(
                            #flush_impl
                        )*
                    }
                }
            }
        }
    });
    Ok(quote! {
        const _: () = {
            impl #impl_generics ink_core::storage::Flush for #ident #type_generics #where_clause {
                fn flush(&mut self) {
                    match self {
                        #( #flush_impl )*
                    }
                }
            }
        };
    })
}
