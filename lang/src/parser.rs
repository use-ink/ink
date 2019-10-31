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

use crate::ast;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    self,
    parse::{
        Parse,
        ParseStream,
        Result,
    },
    Token,
};

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
            .first()
            .expect("paths have at least one segment");
        if let Some(colon) = first_segment.punct() {
            return Err(syn::Error::new(colon.spans[0], "expected meta value"))
        }
        let ident = first_segment.value().ident.clone();
        let parser = |input: ParseStream<'_>| {
            let eq_token = input.parse()?;
            let ty = input.parse()?;
            Ok(Self {
                ident,
                eq_token,
                ty,
            })
        };
        syn::parse::Parser::parse2(parser, attr.tts.clone())
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
            decl: ast::FnDecl {
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
            sig: ast::MethodSig {
                ident,
                decl: ast::FnDecl {
                    fn_tok,
                    paren_tok,
                    inputs,
                    output,
                    generics: syn::Generics {
                        where_clause,
                        ..generics
                    },
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
            if ahead.call(ast::FnArg::arg_self_ref).is_ok() && !ahead.peek(Token![:]) {
                return input
                    .call(ast::FnArg::arg_self_ref)
                    .map(ast::FnArg::SelfRef)
            }
        }

        if input.peek(Token![mut]) || input.peek(Token![self]) {
            let ahead = input.fork();
            if ahead.call(ast::FnArg::arg_self).is_ok() && !ahead.peek(Token![:]) {
                return input.call(ast::FnArg::arg_self).map(ast::FnArg::SelfValue)
            }
        }

        let ahead = input.fork();
        let err = match ahead.call(ast::FnArg::arg_captured) {
            Ok(_) => {
                return input
                    .call(ast::FnArg::arg_captured)
                    .map(ast::FnArg::Captured)
            }
            Err(err) => err,
        };

        Err(err)
    }
}

impl ast::FnArg {
    fn arg_self_ref(input: ParseStream) -> Result<syn::ArgSelfRef> {
        Ok(syn::ArgSelfRef {
            and_token: input.parse()?,
            lifetime: input.parse()?,
            mutability: input.parse()?,
            self_token: input.parse()?,
        })
    }

    fn arg_self(input: ParseStream) -> Result<syn::ArgSelf> {
        Ok(syn::ArgSelf {
            mutability: input.parse()?,
            self_token: input.parse()?,
        })
    }

    fn arg_captured(input: ParseStream) -> Result<syn::ArgCaptured> {
        Ok(syn::ArgCaptured {
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
