use crate::hir2::ItemMeta;
use core::convert::TryFrom;
use trybuild;

#[test]
fn parse_meta_storage() {
    let input: syn::Attribute = syn::parse_quote! { #[ink(storage)] };
    let result = ItemMeta::try_from(input);
    assert!(result.is_ok());
    assert!(result.unwrap().is_simple("storage"));
}

#[test]
fn parse_meta_event() {
    let input: syn::Attribute = syn::parse_quote! { #[ink(event)] };
    let result = ItemMeta::try_from(input);
    assert!(result.is_ok());
    assert!(result.unwrap().is_simple("event"));
}
