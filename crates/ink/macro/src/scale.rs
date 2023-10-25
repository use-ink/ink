// Copyright (C) Parity Technologies (UK) Ltd.
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

pub fn derive(attr: TokenStream2, item: TokenStream2) -> syn::Result<TokenStream2> {
    let mut encode = false;
    let mut decode = false;
    let mut type_info = false;

    syn::parse::Parser::parse2(
        syn::meta::parser(|meta| {
            if meta.path.is_ident("Encode") {
                encode = true;
                Ok(())
            } else if meta.path.is_ident("Decode") {
                decode = true;
                Ok(())
            } else if meta.path.is_ident("TypeInfo") {
                type_info = true;
                Ok(())
            } else {
                Err(meta.error(
                    "unsupported scale derive: expected Encode, Decode or TypeInfo",
                ))
            }
        }),
        attr,
    )?;

    let codec_crate =
        (encode || decode).then(|| quote::quote!(#[codec(crate = ::ink::scale)]));
    let encode = encode.then(|| quote::quote!(#[derive(::ink::scale::Encode)]));
    let decode = decode.then(|| quote::quote!(#[derive(::ink::scale::Decode)]));

    let type_info = type_info.then(|| {
        quote::quote!(
            #[cfg_attr(
                feature = "std",
                derive(::ink::scale_info::TypeInfo),
                scale_info(crate = ::ink::scale_info)
            )]
        )
    });

    Ok(quote::quote!(
        #encode
        #decode
        #codec_crate
        #type_info
        #item
    ))
}
