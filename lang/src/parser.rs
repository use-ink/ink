use crate::proc_macro;

// use proc_macro2::TokenStream;
use crate::ast;
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
}

pub fn parse_contract(token_stream: proc_macro::TokenStream) -> Result<ast::Contract> {
    syn::parse(token_stream).map_err(|e| e.into())
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
        let attrs = syn::Attribute::parse_outer(input)?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            input.parse().map(|mut state: ast::ItemState| {
                state.attrs = attrs;
                ast::Item::State(state)
            })
        } else if lookahead.peek(Token![impl ]) {
            if input.peek2(keywords::Deploy) {
                input.parse().map(|mut deploy_impl: ast::ItemDeployImpl| {
                    deploy_impl.attrs = attrs;
                    ast::Item::DeployImpl(deploy_impl)
                })
            } else {
                input.parse().map(|mut impl_block: ast::ItemImpl| {
                    impl_block.attrs = attrs;
                    ast::Item::Impl(impl_block)
                })
            }
        } else {
            Err(lookahead.error())
        }
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
        let mut content;
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
