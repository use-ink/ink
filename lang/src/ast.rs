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

use crate::parser::keywords;

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

#[allow(clippy::large_enum_variant)]
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
        self.attrs.iter().any(|attr| {
            attr.style == syn::AttrStyle::Outer
                && attr.path.is_ident("indexed")
                && attr.tokens.is_empty()
        })
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
    pub sig: Signature,
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
    pub sig: Signature,
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
pub struct Signature {
    pub ident: Ident,
    pub fn_tok: token::Fn,
    pub generics: syn::Generics,
    pub paren_tok: token::Paren,
    pub inputs: Punctuated<FnArg, token::Comma>,
    pub output: ReturnType,
}

pub struct FnInputs {
    punct: Punctuated<FnArg, Token![,]>,
}

impl FnInputs {
    pub fn to_actual_params(&self) -> Punctuated<syn::Pat, Token![,]> {
        let mut params: Punctuated<syn::Pat, Token![,]> = Default::default();
        for pat_ty in self.punct.iter().filter_map(|fn_arg| {
            if let FnArg::Typed(pat_ty) = fn_arg {
                Some(pat_ty)
            } else {
                None
            }
        }) {
            params.push((*pat_ty.pat).clone())
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

impl Signature {
    pub fn kind(&self) -> FnDeclKind {
        match self.inputs.iter().next().unwrap() {
            FnArg::Receiver(syn::Receiver {
                mutability: Some(_),
                reference: Some(_),
                ..
            }) => FnDeclKind::SelfRefMut,
            FnArg::Receiver(syn::Receiver {
                mutability: None,
                reference: Some(_),
                ..
            }) => FnDeclKind::SelfRef,
            FnArg::Receiver(syn::Receiver {
                mutability: None,
                reference: None,
                ..
            })
            | FnArg::Receiver(syn::Receiver {
                mutability: Some(_),
                reference: None,
                ..
            }) => FnDeclKind::SelfVal,
            _ => FnDeclKind::Static,
        }
    }

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
        let mut inputs_without_self: Punctuated<FnArg, Token![,]> = Default::default();
        for input in self.inputs.iter().skip(1) {
            inputs_without_self.push(input.clone())
        }
        FnInputs {
            punct: inputs_without_self,
        }
    }

    pub fn inputs_with_env(&self, env_handler: &syn::Type) -> FnInputs {
        assert!(self.is_self_ref());
        let mut inputs_with_env: Punctuated<FnArg, Token![,]> = Default::default();
        let mut inputs_iter = self.inputs.iter();
        let self_arg = inputs_iter.next().unwrap();
        inputs_with_env.push_value(self_arg.clone());
        inputs_with_env.push_punct(Default::default());
        let custom_pat_ty: PatType = if self.kind() == FnDeclKind::SelfRefMut {
            syn::parse_quote! { env: &mut #env_handler }
        } else {
            syn::parse_quote! { env: &#env_handler }
        };
        inputs_with_env.push(FnArg::Typed(custom_pat_ty.into_pat_type()));
        for input in inputs_iter {
            inputs_with_env.push(input.clone())
        }
        FnInputs {
            punct: inputs_with_env,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum FnArg {
    Receiver(syn::Receiver),
    Typed(syn::PatType),
}

impl FnArg {
    /// Returns the ident if available.
    pub fn ident(&self) -> Option<proc_macro2::Ident> {
        match self {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_ty) => {
                match &*pat_ty.pat {
                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                    _ => None,
                }
            }
        }
    }

    /// Returns `true` if the fn argument is accepted by pattern and type.
    pub fn is_typed(&self) -> Option<&syn::PatType> {
        match self {
            FnArg::Typed(pat_ty) => Some(pat_ty),
            _ => None,
        }
    }
}

impl quote::ToTokens for FnArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            FnArg::Receiver(receiver) => receiver.to_tokens(tokens),
            FnArg::Typed(pat_ty) => pat_ty.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
pub struct PatType {
    pub attrs: Vec<syn::Attribute>,
    pub pat: Box<syn::Pat>,
    pub colon_token: Token![:],
    pub ty: Box<syn::Type>,
}

impl syn::parse::Parse for PatType {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::parse::Result<Self> {
        let attrs = syn::Attribute::parse_outer(input)?;
        let pat = input.parse()?;
        let colon_token = input.parse()?;
        let ty = input.parse()?;
        Ok(Self {
            attrs,
            pat,
            colon_token,
            ty,
        })
    }
}

impl PatType {
    pub fn into_pat_type(self) -> syn::PatType {
        syn::PatType {
            attrs: self.attrs,
            pat: self.pat,
            colon_token: self.colon_token,
            ty: self.ty,
        }
    }
}
