use ir::{
    Callable,
    CallableWithSelector,
    Message,
};
use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
};
use quote::{
    format_ident,
    quote,
};
use syn::Type;

/// Returns Solidity ABI compatible selector of an ink! message.
pub(crate) fn solidity_selector(message: &CallableWithSelector<Message>) -> TokenStream2 {
    let call_type_ident = solidity_call_type_ident(message);
    quote!(
        {
            <__ink_sol_interop__::#call_type_ident>::SELECTOR
        }
    )
}

/// Returns a `u32` representation of the Solidity ABI compatible selector of an ink!
/// message.
pub(crate) fn solidity_selector_id(
    message: &CallableWithSelector<Message>,
) -> TokenStream2 {
    let call_type_ident = solidity_call_type_ident(message);
    quote!(
        {
            <__ink_sol_interop__::#call_type_ident>::SELECTOR_ID
        }
    )
}

/// Returns the Solidity call info type identifier for an ink! message.
///
/// See [`ink::alloy_sol_types::sol`]
pub(crate) fn solidity_call_type_ident(message: &CallableWithSelector<Message>) -> Ident {
    let ident = message.ident();
    let id = message.composed_selector().into_be_u32();
    format_ident!("{ident}{id}Call")
}

/// Returns [`ink::alloy_sol_types::SolType`] representation for the given type.
pub(crate) fn sol_type(ty: &Type) -> TokenStream2 {
    match ty {
        // TODO: (@davidsemakula) replace with more robust solution before release v6
        // release. Necessary because `alloy_sol_types::SolValue` is not
        // implemented for u8.
        Type::Path(ty) if ty.path.is_ident("u8") => {
            quote! {
                ::ink::alloy_sol_types::sol_data::Uint<8>
            }
        }
        Type::Reference(ty) => sol_type(&ty.elem),
        Type::Tuple(tys) => {
            let tuple_tys = tys.elems.iter().map(sol_type);
            quote! {
                (#(#tuple_tys,)*)
            }
        }
        _ => {
            quote! {
                <#ty as ::ink::alloy_sol_types::SolValue>::SolType
            }
        }
    }
}
