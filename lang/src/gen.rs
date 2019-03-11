use crate::{
    ast,
    hir,
};
use proc_macro2::{
    Ident,
    Span,
    TokenStream,
};
use quote::{
    quote,
    ToTokens,
};
use syn::{
    punctuated::Punctuated,
    Token,
};

pub fn codegen(contract: &hir::Contract) -> proc_macro2::TokenStream {
    let mut tokens = quote! {};
    codegen_for_state(&mut tokens, contract);
    codegen_for_messages(&mut tokens, contract);
    tokens
}

fn codegen_for_state(tokens: &mut TokenStream, contract: &hir::Contract) {
    let state_attrs_toks = {
        let mut content = quote! {};
        for attr in &contract.state.attrs {
            attr.to_tokens(&mut content)
        }
        content
    };
    let struct_fields_toks = &contract.state.fields;
    let name = &contract.name;
    tokens.extend(quote! {
        pdsl_model::state! {
            #state_attrs_toks
            struct #name
                #struct_fields_toks
        }
    });
}

fn codegen_for_messages(tokens: &mut TokenStream, contract: &hir::Contract) {
    let messages_content = {
        let mut content = quote! {};
        for (n, message) in contract.messages.iter().enumerate() {
            for attr in &message.attrs {
                attr.to_tokens(&mut content)
            }
            let msg_id =
                syn::LitInt::new(n as u64, syn::IntSuffix::None, Span::call_site());
            msg_id.to_tokens(&mut content);
            <Token![=>]>::default().to_tokens(&mut content);
            use crate::ident_ext::IdentExt as _;
            use heck::CamelCase as _;
            let camel_case_ident = Ident::new(
                &message.sig.ident.to_owned_string().to_camel_case(),
                message.sig.ident.span(),
            );
            camel_case_ident.to_tokens(&mut content);
            let fn_decl = &message.sig.decl;
            fn_decl.paren_tok.surround(&mut content, |inner_toks| {
                let args_without_self = {
                    let mut args_without_self: Punctuated<ast::FnArg, Token![,]> =
                        Punctuated::new();
                    for fn_arg in fn_decl
							.inputs.iter()
							// Performing `skip(1)` here works because we already asserted
							// that all messages have to start with either `&self` or `&mut self`.
							.skip(1)
                    {
                        args_without_self.push(fn_arg.clone())
                    }
                    args_without_self
                };
                args_without_self.to_tokens(inner_toks)
            });
            fn_decl.output.to_tokens(&mut content);
            <Token![;]>::default().to_tokens(&mut content);
        }
        content
    };
    tokens.extend(quote! {
		// Apparently this `use` is required even though it should not be.
		// -> Further investigations needed!
        use pdsl_model::messages;
        pdsl_model::messages! {
            #messages_content
        }
    })
}
