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
    Span,
    TokenStream as TokenStream2,
};
use syn::{
    parse::{
        Parse,
        ParseStream,
        Result,
    },
    Token,
};

use crate::ast;

pub mod keywords {
    use syn::custom_keyword;

    custom_keyword!(Deploy);
    custom_keyword!(deploy);
    custom_keyword!(external);
    custom_keyword!(event);
}

pub fn parse_contract(token_stream: TokenStream2) -> Result<ast::Contract> {
    syn::parse2(token_stream).map_err(Into::into)
}

impl Parse for ast::Contract {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(ast::Contract {
            items: ast::Item::parse_outer(input)?,
        })
    }
}

impl ast::Item {
    fn parse_outer(input: ParseStream<'_>) -> Result<Vec<Self>> {
        let mut res = Vec::new();
        while !input.is_empty() {
            res.push(input.parse()?);
        }
        Ok(res)
    }
}

impl Parse for ast::Item {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let inner_attrs: ast::ItemEnvMeta = input.parse()?;
        if !inner_attrs.env_types_metas.is_empty() {
            return Ok(ast::Item::EnvMeta(inner_attrs))
        }
        let attrs_outer = syn::Attribute::parse_outer(input)?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            input.parse().map(|mut state: ast::ItemState| {
                state.attrs = attrs_outer;
                ast::Item::State(state)
            })
        } else if lookahead.peek(Token![impl]) {
            if input.peek2(keywords::Deploy) {
                input.parse().map(|mut deploy_impl: ast::ItemDeployImpl| {
                    deploy_impl.attrs = attrs_outer;
                    ast::Item::DeployImpl(deploy_impl)
                })
            } else {
                input.parse().map(|mut impl_block: ast::ItemImpl| {
                    impl_block.attrs = attrs_outer;
                    ast::Item::Impl(impl_block)
                })
            }
        } else if lookahead.peek(keywords::event) {
            input.parse().map(|mut event: ast::ItemEvent| {
                event.attrs = attrs_outer;
                ast::Item::Event(event)
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for ast::ItemEnvMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_inner)?;
        let env_types = attrs
            .iter()
            .map(ast::ItemEnvTypesMeta::parse_from_attr)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            env_types_metas: env_types,
        })
    }
}

impl ast::ItemEnvTypesMeta {
    fn parse_from_attr(attr: &syn::Attribute) -> Result<Self> {
        let first_segment = attr
            .path
            .segments
            .pairs()
            .next()
            .expect("paths have at least one segment")
            .into_tuple();
        if let Some(colon) = first_segment.1 {
            return Err(syn::Error::new(colon.spans[0], "expected meta value"))
        }
        let ident = first_segment.0.ident.clone();
        let parser = |input: ParseStream<'_>| {
            let eq_token = input.parse()?;
            let ty = input.parse()?;
            Ok(Self {
                ident,
                eq_token,
                ty,
            })
        };
        syn::parse::Parser::parse2(parser, attr.tokens.clone())
    }
}

impl Parse for ast::ItemState {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let struct_tok = input.parse()?;
        let ident = input.parse()?;
        let fields = input.parse()?;
        Ok(Self {
            attrs: vec![],
            struct_tok,
            ident,
            fields,
        })
    }
}

impl Parse for ast::ItemDeployImpl {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let impl_tok = input.parse()?;
        let deploy_tok = input.parse()?;
        let for_tok = input.parse()?;
        let self_ty = input.parse()?;
        let content;
        let (brace_tok, inner_attrs, deploy_fn_impl) = {
            let brace_tok = syn::braced!(content in input);
            let inner_attrs = content.call(syn::Attribute::parse_inner)?;
            let deploy_fn_impl = content.parse()?;
            (brace_tok, inner_attrs, deploy_fn_impl)
        };
        Ok(Self {
            attrs: inner_attrs,
            impl_tok,
            deploy_tok,
            for_tok,
            self_ty,
            brace_tok,
            item: deploy_fn_impl,
        })
    }
}

impl Parse for ast::DeployItemMethod {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = syn::Attribute::parse_outer(input)?;
        let fn_tok = input.parse()?;
        let deploy_tok = input.parse()?;
        let (paren_tok, inputs) = {
            let content;
            let paren_tok = syn::parenthesized!(content in input);
            let inputs = content.parse_terminated(ast::FnArg::parse)?;
            (paren_tok, inputs)
        };
        let output = input.parse()?;
        let block = input.parse()?;
        Ok(Self {
            attrs,
            deploy_tok,
            sig: ast::Signature {
                ident: Ident::new("deploy", Span::call_site()),
                fn_tok,
                paren_tok,
                inputs,
                output,
                generics: Default::default(),
            },
            block,
        })
    }
}

impl Parse for ast::ItemImpl {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let impl_tok = input.parse()?;
        let self_ty = input.parse()?;
        let (brace_tok, inner_attrs, items) = {
            let content;
            let brace_tok = syn::braced!(content in input);
            let inner_attrs = content.call(syn::Attribute::parse_inner)?;

            let mut items = Vec::new();
            while !content.is_empty() {
                items.push(content.parse()?);
            }
            (brace_tok, inner_attrs, items)
        };
        Ok(Self {
            attrs: inner_attrs,
            impl_tok,
            self_ty,
            brace_tok,
            items,
        })
    }
}

impl Parse for ast::ItemImplMethod {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = syn::Attribute::parse_outer(input)?;
        let vis = input.parse()?;
        let fn_tok = input.parse()?;
        let ident = input.parse()?;
        let generics: syn::Generics = input.parse()?;
        let (paren_tok, inputs) = {
            let content;
            let paren_tok = syn::parenthesized!(content in input);
            let inputs = content.parse_terminated(ast::FnArg::parse)?;
            (paren_tok, inputs)
        };
        let output = input.parse()?;
        let where_clause: Option<syn::WhereClause> = input.parse()?;
        let block = input.parse()?;

        Ok(Self {
            attrs,
            vis,
            sig: ast::Signature {
                ident,
                fn_tok,
                paren_tok,
                inputs,
                output,
                generics: syn::Generics {
                    where_clause,
                    ..generics
                },
            },
            block,
        })
    }
}

impl Parse for ast::MethodVisibility {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Token![pub]) {
            Ok(ast::MethodVisibility::External(input.parse()?))
        } else {
            Ok(ast::MethodVisibility::Inherited)
        }
    }
}

impl Parse for ast::ExternalVisibility {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let pub_tok = input.parse::<Token![pub]>()?;
        let content;
        let paren_tok = syn::parenthesized!(content in input);
        let external_tok = content.parse()?;
        Ok(ast::ExternalVisibility {
            pub_tok,
            paren_tok,
            external_tok,
        })
    }
}

impl Parse for ast::FnArg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Token![&]) {
            let ahead = input.fork();
            if ahead.call(ast::FnArg::receiver).is_ok() && !ahead.peek(Token![:]) {
                return input.call(ast::FnArg::receiver).map(ast::FnArg::Receiver)
            }
        }

        if input.peek(Token![mut]) || input.peek(Token![self]) {
            let ahead = input.fork();
            if ahead.call(ast::FnArg::receiver).is_ok() && !ahead.peek(Token![:]) {
                return input.call(ast::FnArg::receiver).map(ast::FnArg::Receiver)
            }
        }

        let ahead = input.fork();
        let err = match ahead.call(ast::FnArg::pat_typed) {
            Ok(_) => return input.call(ast::FnArg::pat_typed).map(ast::FnArg::Typed),
            Err(err) => err,
        };

        Err(err)
    }
}

impl ast::FnArg {
    fn receiver(input: ParseStream<'_>) -> Result<syn::Receiver> {
        Ok(input.parse()?)
    }

    fn pat_typed(input: ParseStream<'_>) -> Result<syn::PatType> {
        Ok(syn::PatType {
            attrs: syn::Attribute::parse_outer(input)?,
            pat: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

impl Parse for ast::ItemEvent {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let event_tok = input.parse()?;
        let ident = input.parse()?;
        let (brace_tok, args) = {
            let content;
            let brace_tok = syn::braced!(content in input);
            let inputs = content.parse_terminated(ast::EventArg::parse)?;
            (brace_tok, inputs)
        };
        Ok(Self {
            attrs: vec![],
            event_tok,
            ident,
            brace_tok,
            args,
        })
    }
}

impl Parse for ast::EventArg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = syn::Attribute::parse_outer(input)?;
        let ident = input.parse()?;
        let colon_tok = input.parse()?;
        let ty = input.parse()?;
        Ok(Self {
            attrs,
            ident,
            colon_tok,
            ty,
        })
    }
}
