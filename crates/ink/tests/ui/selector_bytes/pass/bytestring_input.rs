use ink_ir as ir;

fn main() {
    // We put it into a constant to verify that the computation is constant.
    const HASH1: [u8; 4] = ink::selector_bytes!(Abi::Ink, b"");
    assert_eq!(
        HASH1,
        ir::Selector::compute(b"", ir::SelectorAbi::Ink).to_bytes(),
    );

    const HASH2: [u8; 4] = ink::selector_bytes!(Abi::Ink, b"Hello, World!");
    assert_eq!(
        HASH2,
        ir::Selector::compute(b"Hello, World!", ir::SelectorAbi::Ink).to_bytes(),
    );

    const HASH3: [u8; 4] = ink::selector_bytes!(Abi::Ink, b"message");
    assert_eq!(
        HASH3,
        ir::Selector::compute(b"message", ir::SelectorAbi::Ink).to_bytes(),
    );

    const HASH4: [u8; 4] = ink::selector_bytes!(Abi::Ink, b"constructor");
    assert_eq!(
        HASH4,
        ir::Selector::compute(b"constructor", ir::SelectorAbi::Ink).to_bytes(),
    );
}
