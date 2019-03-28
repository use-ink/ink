// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use crate::parser::keywords;

use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
};
use syn::{
    punctuated::Punctuated,
    token,
    ReturnType,
    Token,
};

#[derive(Debug)]
pub struct Contract {
    pub items: Vec<Item>,
}

impl Contract {
    pub fn states<'a>(&'a self) -> impl Iterator<Item = &'a ItemState> + 'a {
        self.items.iter().filter_map(|item| {
            match *item {
                Item::State(ref c) => Some(c),
                _ => None,
            }
        })
    }

    pub fn deploy_impl_blocks<'a>(
        &'a self,
    ) -> impl Iterator<Item = &'a ItemDeployImpl> + 'a {
        self.items.iter().filter_map(|item| {
            match *item {
                Item::DeployImpl(ref d) => Some(d),
                _ => None,
            }
        })
    }

    pub fn impl_blocks<'a>(&'a self) -> impl Iterator<Item = &'a ItemImpl> + 'a {
        self.items.iter().filter_map(|item| {
            match *item {
                Item::Impl(ref i) => Some(i),
                _ => None,
            }
        })
    }
}

#[derive(Debug)]
pub enum Item {
    State(ItemState),
    DeployImpl(ItemDeployImpl),
    Impl(ItemImpl),
}

#[derive(Debug)]
pub struct ItemState {
    pub attrs: Vec<syn::Attribute>,
    pub struct_tok: token::Struct,
    pub ident: Ident,
    pub fields: syn::FieldsNamed,
}

#[derive(Debug)]
pub struct ItemDeployImpl {
    pub attrs: Vec<syn::Attribute>,
    pub impl_tok: Token![impl ],
    pub deploy_tok: keywords::Deploy,
    pub for_tok: Token![for],
    pub self_ty: Ident,
    pub brace_tok: token::Brace,
    pub item: DeployItemMethod,
}

#[derive(Debug)]
pub struct DeployItemMethod {
    pub attrs: Vec<syn::Attribute>,
    pub deploy_tok: keywords::deploy,
    pub decl: FnDecl,
    pub block: syn::Block,
}

#[derive(Debug)]
pub struct ItemImpl {
    pub attrs: Vec<syn::Attribute>,
    pub impl_tok: token::Impl,
    pub self_ty: Ident,
    pub brace_tok: token::Brace,
    pub items: Vec<ItemImplMethod>,
}

#[derive(Debug)]
pub struct ItemImplMethod {
    pub attrs: Vec<syn::Attribute>,
    pub vis: MethodVisibility,
    pub sig: MethodSig,
    pub block: syn::Block,
}

#[derive(Debug, Clone)]
pub enum MethodVisibility {
    External(ExternalVisibility),
    Inherited,
}

impl MethodVisibility {
    /// Returns `true` if this is an external visibility.
    ///
    /// # Note
    ///
    /// The `pub(external)` visibility is only used for contract messages.
    pub fn is_external(&self) -> bool {
        match self {
            MethodVisibility::External(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExternalVisibility {
    pub pub_tok: token::Pub,
    pub paren_tok: token::Paren,
    pub external_tok: keywords::external,
}

#[derive(Debug, Clone)]
pub struct MethodSig {
    pub ident: Ident,
    pub decl: FnDecl,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub fn_tok: token::Fn,
    pub paren_tok: token::Paren,
    pub inputs: Punctuated<FnArg, token::Comma>,
    pub output: ReturnType,
    pub generics: syn::Generics,
}

#[derive(Debug, Clone)]
pub enum FnArg {
    SelfRef(syn::ArgSelfRef),
    SelfValue(syn::ArgSelf),
    Captured(syn::ArgCaptured),
}

impl quote::ToTokens for FnArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            FnArg::SelfRef(arg_self_ref) => arg_self_ref.to_tokens(tokens),
            FnArg::SelfValue(arg_self_value) => arg_self_value.to_tokens(tokens),
            FnArg::Captured(arg_captured) => arg_captured.to_tokens(tokens),
        }
    }
}
