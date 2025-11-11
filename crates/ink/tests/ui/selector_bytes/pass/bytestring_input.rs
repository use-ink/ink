use ink_ir as ir;

macro_rules! assert_macro_eq {
    ( $input:literal ) => {{
        // We put it into a constant to verify that the computation is constant.
        const HASH: [u8; 4] = ink::selector_bytes!(Abi::Ink, Abi::Ink, $input);
        assert_eq!(
            HASH,
            ir::Selector::compute($input, ir::SelectorAbi::Ink).to_bytes(),
        );
    }};
}

fn main() {
    assert_macro_eq!(b"");
    assert_macro_eq!(b"Hello, World!");
    assert_macro_eq!(b"message");
    assert_macro_eq!(b"constructor");
}
