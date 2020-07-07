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
    GenerateCodeUsing as _,
};
use derive_more::From;
use ir::Callable as _;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
    ToTokens,
};
use syn::spanned::Spanned as _;

/// Generates code for all ink! implementation blocks.
#[derive(From)]
pub struct ItemImpls<'a> {
    contract: &'a ir::Contract,
}

impl AsRef<ir::Contract> for ItemImpls<'_> {
    fn as_ref(&self) -> &ir::Contract {
        &self.contract
    }
}

impl GenerateCode for ItemImpls<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let item_impls = self.contract.module().impls().map(Self::generate_item_impl);
        let no_cross_calling_cfg =
            self.generate_code_using::<generator::CrossCallingConflictCfg>();
        quote! {
            #no_cross_calling_cfg
            const _: () = {
                use ::ink_lang::{Env, EmitEvent, StaticEnv};

                #( #item_impls )*
            };
        }
    }
}

impl ItemImpls<'_> {
    /// Generates the code for the given ink! message within an implementation block.
    fn generate_message(message: &ir::Message) -> TokenStream2 {
        let span = message.span();
        let attrs = message.attrs();
        let vis = match message.visibility() {
            ir::Visibility::Inherited => None,
            ir::Visibility::Public(vis_public) => Some(vis_public),
        };
        let receiver = match message.receiver() {
            ir::Receiver::RefMut => quote! { &mut self },
            ir::Receiver::Ref => quote! { &self },
        };
        let ident = message.ident();
        let inputs = message.inputs();
        let output_arrow = message.output().map(|_| quote! { -> });
        let output = message.output();
        let statements = message.statements();
        quote_spanned!(span =>
            #( #attrs )*
            #vis fn #ident(#receiver, #( #inputs ),* ) #output_arrow #output {
                #( #statements )*
            }
        )
    }

    /// Generates the code for the given ink! constructor within an implementation block.
    fn generate_constructor(constructor: &ir::Constructor) -> TokenStream2 {
        let span = constructor.span();
        let attrs = constructor.attrs();
        let vis = match constructor.visibility() {
            ir::Visibility::Inherited => None,
            ir::Visibility::Public(vis_public) => Some(vis_public),
        };
        let ident = constructor.ident();
        let inputs = constructor.inputs();
        let statements = constructor.statements();
        quote_spanned!(span =>
            #( #attrs )*
            #vis fn #ident( #( #inputs ),* ) -> Self {
                #( #statements )*
            }
        )
    }

    /// Generates code for the given ink! implementation block.
    fn generate_item_impl(item_impl: &ir::ItemImpl) -> TokenStream2 {
        let span = item_impl.span();
        let messages = item_impl
            .iter_messages()
            .map(|cws| Self::generate_message(cws.callable()));
        let constructors = item_impl
            .iter_constructors()
            .map(|cws| Self::generate_constructor(cws.callable()));
        let other_items = item_impl
            .items()
            .iter()
            .filter_map(ir::ImplItem::filter_map_other_item)
            .map(ToTokens::to_token_stream);
        let trait_path = item_impl.trait_path();
        let trait_for = item_impl.trait_path().map(|_| quote! { for });
        let self_type = item_impl.self_type();
        quote_spanned!(span =>
            impl #trait_path #trait_for #self_type {
                #( #constructors )*
                #( #messages )*
                #( #other_items )*
            }
        )
    }
}
