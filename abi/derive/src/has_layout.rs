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

use crate::impl_wrapper::wrap;

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
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

    Ok(wrap(has_layout_impl))
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
                _ink_abi::LayoutStruct::new(<Self as type_metadata::Metadata>::meta_type(), Vec::new())
            }
        }
    }
}
