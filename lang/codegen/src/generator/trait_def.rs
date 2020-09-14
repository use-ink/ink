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

use crate::{
    generator,
    GenerateCode,
    GenerateCodeUsing,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

/// Generator to create the ink! storage struct and important trait impls.
#[derive(From)]
pub struct TraitDefinition<'a> {
    trait_def: &'a ir::InkTrait,
}

impl<'a> AsRef<ir::InkTrait> for TraitDefinition<'_> {
    fn as_ref(&self) -> &ir::InkTrait {
        &self.trait_def
    }
}

impl GenerateCode for TraitDefinition<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let span = self.trait_def.span();
        let attrs = self.trait_def.attrs();
        let ident = self.trait_def.ident();
        let verify_hash = self.trait_def.verify_hash();
        let constructors = self
            .trait_def
            .iter_items()
            .flat_map(ir::InkTraitItem::filter_map_constructor)
            .map(|constructor| {
                let span = constructor.span();
                let attrs = constructor.attrs();
                let sig = constructor.sig();
                let ident = &sig.ident;
                let inputs = &sig.inputs;
                quote_spanned!(span =>
                    #(#attrs)*
                    fn #ident(#inputs) -> Self::Output;
                )
            });
        let messages = self
            .trait_def
            .iter_items()
            .flat_map(ir::InkTraitItem::filter_map_message)
            .map(|message| {
                let span = message.span();
                let attrs = message.attrs();
                let sig = message.sig();
                let ident = &sig.ident;
                let inputs = &sig.inputs;
                let output = &sig.output;
                quote_spanned!(span =>
                    #(#attrs)*
                    fn #ident(#inputs) -> #output;
                )
            });
        quote_spanned!(span =>
            #(#attrs)*
            pub trait #ident {
                type Output;

                #(#constructors)*
                #(#messages)*
            }
        )
    }
}
