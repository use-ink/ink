use crate::ir::{
    Marker,
    Params,
};
use core::convert::TryFrom;
use trybuild;

#[test]
fn parse_meta_storage() {
    let input: syn::Attribute = syn::parse_quote! { #[ink(storage)] };
    let result = Marker::try_from(input);
    assert!(result.is_ok());
    assert!(result.unwrap().is_simple("storage"));
}

#[test]
fn parse_meta_event() {
    let input: syn::Attribute = syn::parse_quote! { #[ink(event)] };
    let result = Marker::try_from(input);
    assert!(result.is_ok());
    assert!(result.unwrap().is_simple("event"));
}

#[test]
fn parse_params() {
    let input: Params = syn::parse_quote! {
        env = DefaultSrmlTypes, version = [0, 1, 0]
    };
}
