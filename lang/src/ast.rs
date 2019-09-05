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
    parse::Result,
};

#[derive(Debug)]
pub struct Contract {
    pub items: Vec<Item>,
}

impl Contract {
    pub fn env_metas<'a>(&'a self) -> impl Iterator<Item = &'a ItemEnvMeta> + 'a {
        self.items.iter().filter_map(|item| {
            match *item {
                Item::EnvMeta(ref t) => Some(t),
                _ => None,
            }
        })
    }

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

    pub fn events<'a>(&'a self) -> impl Iterator<Item = &'a ItemEvent> + 'a {
        self.items.iter().filter_map(|item| {
            match *item {
                Item::Event(ref event) => Some(event),
                _ => None,
            }
        })
    }
}

#[derive(Debug)]
pub enum Item {
    EnvMeta(ItemEnvMeta),
    State(ItemState),
    DeployImpl(ItemDeployImpl),
    Impl(ItemImpl),
    Event(ItemEvent),
}

#[derive(Debug, Clone)]
pub struct ItemEnvMeta {
    pub env_types_metas: Vec<ItemEnvTypesMeta>,
}

#[derive(Debug, Clone)]
pub struct ItemEnvTypesMeta {
    pub ident: Ident,
    pub eq_token: Token![=],
    pub ty: syn::Type,
}

/// An event declaration.
///
/// # Example
///
/// This mirrors the syntax for: `event Foo { bar: Bar };`
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ItemEvent {
    pub attrs: Vec<syn::Attribute>,
    pub event_tok: crate::parser::keywords::event,
    pub ident: Ident,
    pub brace_tok: token::Brace,
    pub args: Punctuated<EventArg, token::Comma>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventArg {
    pub attrs: Vec<syn::Attribute>,
    pub ident: Ident,
    pub colon_tok: Token![:],
    pub ty: syn::Type,
}

impl EventArg {
    /// Returns `true` if the event argument is indexed.
    pub fn is_indexed(&self) -> bool {
        self.attrs
            .iter()
            .find(|attr| {
                attr.style == syn::AttrStyle::Outer
                    && attr.path.is_ident("indexed")
                    && attr.tokens.is_empty()
            })
            .is_some()
    }
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
    pub impl_tok: Token![impl],
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
    pub inputs: Punctuated<syn::FnArg, token::Comma>,
    pub output: ReturnType,
    pub generics: syn::Generics,
}

pub struct FnInputs {
    punct: Punctuated<syn::FnArg, Token![,]>,
}

impl FnInputs {
    pub fn to_actual_params(&self) -> Punctuated<syn::Pat, Token![,]> {
        let mut params: Punctuated<syn::Pat, Token![,]> = Default::default();
        for pat_type in self.punct.iter().filter_map(|fn_arg| {
            if let syn::FnArg::Typed(pat_type) = fn_arg {
                Some(pat_type)
            } else {
                None
            }
        }) {
            params.push((*pat_type.pat).clone())
        }
        params
    }
}

impl quote::ToTokens for FnInputs {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.punct.to_tokens(tokens)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FnDeclKind {
    SelfRef,
    SelfRefMut,
    SelfVal,
    Static,
}

impl FnDecl {
    // pub fn kind(&self) -> FnDeclKind {
    //     match self.inputs.iter().next().unwrap() {
    //         syn::FnArg::Receiver(receiver) => {
    //             if receiver.reference.is_none() {
    //                 bail!(receiver.span(), "receiver must be a reference")
    //             }
    //             if receiver.mutability.is_some() {
    //                 FnDeclKind::SelfRefMut
    //             } else {
    //                 FnDeclKind::SelfRef
    //             }
    //         }
    //         syn::FnArg::SelfValue(_) => FnDeclKind::SelfVal,
    //         _ => FnDeclKind::Static,
    //     }
    // }

    pub fn is_self_ref(&self) -> bool {
        if let FnDeclKind::SelfRef | FnDeclKind::SelfRefMut = self.kind() {
            return true
        }
        false
    }

    pub fn inputs(&self) -> FnInputs {
        assert!(self.is_self_ref());
        FnInputs {
            punct: self.inputs.clone(),
        }
    }

    pub fn inputs_without_self(&self) -> FnInputs {
        assert!(self.is_self_ref());
        let mut inputs_without_self: Punctuated<syn::FnArg, Token![,]> = Default::default();
        for input in self.inputs.iter().skip(1) {
            inputs_without_self.push(input.clone())
        }
        FnInputs {
            punct: inputs_without_self,
        }
    }

    pub fn inputs_with_env(&self, env_types: &syn::Type) -> FnInputs {
        assert!(self.is_self_ref());
        let mut inputs_with_env: Punctuated<syn::FnArg, Token![,]> = Default::default();
        let mut inputs_iter = self.inputs.iter();
        let self_arg = inputs_iter.next().unwrap();
        inputs_with_env.push_value(self_arg.clone());
        inputs_with_env.push_punct(Default::default());
        let custom_arg_captured: syn::PatType = if self.kind() == FnDeclKind::SelfRefMut {
            syn::parse_quote! {
                env: &mut ink_model::EnvHandler<ink_core::env::ContractEnv<#env_types>>
            }
        } else {
            syn::parse_quote! {
                env: &ink_model::EnvHandler<ink_core::env::ContractEnv<#env_types>>
            }
        };
        inputs_with_env.push(syn::FnArg::Captured(custom_arg_captured.into_arg_captured()));
        for input in inputs_iter {
            inputs_with_env.push(input.clone())
        }
        FnInputs {
            punct: inputs_with_env,
        }
    }
}
