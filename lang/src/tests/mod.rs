mod utils;
mod noop;

pub(crate) use quote::quote;
pub(crate) use utils::assert_eq_tokenstreams;
pub(crate) use crate::contract_gen_impl2;

#[test]
fn empty_contract_input() {
    assert!(contract_gen_impl2(quote!{}).is_err());
}
