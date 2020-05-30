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

use quote::quote;
use proc_macro2::TokenStream as TokenStream2;

fn storage_layout_struct(_s: &synstructure::Structure) -> TokenStream2 {
    quote! {}
}

fn storage_layout_enum(_s: &synstructure::Structure) -> TokenStream2 {
    quote! {}
}

pub fn storage_layout_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::Move);
    s.add_bounds(synstructure::AddBounds::Generics);
    match s.ast().data {
        syn::Data::Struct(_) => storage_layout_struct(&s),
        syn::Data::Enum(_) => storage_layout_enum(&s),
        _ => panic!("cannot derive `StorageLayout` for Rust `union` items")
    }
}
