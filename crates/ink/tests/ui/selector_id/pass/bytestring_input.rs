use ink_ir as ir;

fn main() {
    // We put it into a constant to verify that the computation is constant.
    const HASH1: u32 = ink::selector_id!(Abi::Ink, b"");
    assert_eq!(
        HASH1,
        ir::Selector::compute(b"", ir::SelectorAbi::Ink).into_be_u32(),
    );

    const HASH2: u32 = ink::selector_id!(Abi::Ink, b"Hello, World!");
    assert_eq!(
        HASH2,
        ir::Selector::compute(b"Hello, World!", ir::SelectorAbi::Ink).into_be_u32(),
    );

    const HASH3: u32 = ink::selector_id!(Abi::Ink, b"message");
    assert_eq!(
        HASH3,
        ir::Selector::compute(b"message", ir::SelectorAbi::Ink).into_be_u32(),
    );

    const HASH4: u32 = ink::selector_id!(Abi::Ink, b"constructor");
    assert_eq!(
        HASH4,
        ir::Selector::compute(b"constructor", ir::SelectorAbi::Ink).into_be_u32(),
    );
}
