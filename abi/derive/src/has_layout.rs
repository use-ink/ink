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

use crate::impl_wrapper::wrap;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    self,
    parse::Result,
    parse_quote,
    punctuated::Punctuated,
    Data,
    DataStruct,
    DeriveInput,
    Field,
    Fields,
    Token,
};

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_impl(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

pub fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
    let mut ast: DeriveInput = syn::parse2(input)?;

    ast.generics.type_params_mut().for_each(|p| {
        p.bounds.push(parse_quote!(_ink_abi::HasLayout));
    });

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let layout = match &ast.data {
        Data::Struct(ref s) => generate_struct_layout(s),
        Data::Enum(ref _e) => bail!(&ast, "enums are not supported"),
        Data::Union(ref _u) => bail!(&ast, "unions are not supported"),
    };

    let has_layout_impl = quote! {
        impl #impl_generics _ink_abi::HasLayout for #ident #ty_generics #where_clause {
            fn layout(&self) -> _ink_abi::StorageLayout {
                #layout.into()
            }
        }
    };

    Ok(wrap(ident, "HAS_LAYOUT", has_layout_impl).into())
}

fn generate_fields_layout<'a>(
    fields: &'a Punctuated<Field, Token![,]>,
) -> impl Iterator<Item = TokenStream2> + 'a {
    fields.iter().enumerate().map(|(n, field)| {
        let ident = &field.ident;
        if let Some(ident) = ident {
            quote! {
                _ink_abi::LayoutField::new(stringify!(#ident), self.#ident.layout())
            }
        } else {
            let n = proc_macro2::Literal::usize_unsuffixed(n);
            quote! {
                _ink_abi::LayoutField::new(stringify!(#n), self.#n.layout())
            }
        }
    })
}

fn generate_struct_fields_layout(fields: &Punctuated<Field, Token![,]>) -> TokenStream2 {
    let fields_layout = generate_fields_layout(fields);
    quote! {
        use type_metadata::Metadata as _;
        _ink_abi::LayoutStruct::new(Self::meta_type(), __core::vec![
            #( #fields_layout, )*
        ])
    }
}

fn generate_struct_layout(data_struct: &DataStruct) -> TokenStream2 {
    match data_struct.fields {
        Fields::Named(ref fs) => generate_struct_fields_layout(&fs.named),
        Fields::Unnamed(ref fs) => generate_struct_fields_layout(&fs.unnamed),
        Fields::Unit => {
            quote! {
                use type_metadata::Metadata as _;
                _ink_abi::LayoutStruct::new(Self::meta_type(), __core::vec![])
            }
        }
    }
}
