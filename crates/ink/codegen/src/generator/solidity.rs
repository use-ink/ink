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
pub(crate) fn solidity_call_type_ident(message: &CallableWithSelector<Message>) -> Ident {
    let ident = message.ident();
    let id = message.composed_selector().into_be_u32();
    format_ident!("{ident}{id}Call")
}
