use ink_ir as ir;

fn main() {
    // We put it into a constant to verify that the computation is constant.
    const HASH1: [u8; 4] = ink::selector_bytes!(Abi::Ink, "");
    assert_eq!(
        HASH1,
        ir::Selector::compute("".as_bytes(), ir::SelectorAbi::Ink).to_bytes(),
    );

    const HASH2: [u8; 4] = ink::selector_bytes!(Abi::Ink, "Hello, World!");
    assert_eq!(
        HASH2,
        ir::Selector::compute("Hello, World!".as_bytes(), ir::SelectorAbi::Ink).to_bytes(),
    );

    const HASH3: [u8; 4] = ink::selector_bytes!(Abi::Ink, "message");
    assert_eq!(
        HASH3,
        ir::Selector::compute("message".as_bytes(), ir::SelectorAbi::Ink).to_bytes(),
    );

    const HASH4: [u8; 4] = ink::selector_bytes!(Abi::Ink, "constructor");
    assert_eq!(
        HASH4,
        ir::Selector::compute("constructor".as_bytes(), ir::SelectorAbi::Ink).to_bytes(),
    );
}
