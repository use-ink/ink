use ink_ir as ir;

fn main() {
    // We put it into a constant to verify that the computation is constant.
    const HASH1: u32 = ink::selector_id!(Abi::Ink, "");
    assert_eq!(
        HASH1,
        ir::Selector::compute("".as_bytes(), ir::SelectorAbi::Ink).into_be_u32(),
    );

    const HASH2: u32 = ink::selector_id!(Abi::Ink, "Hello, World!");
    assert_eq!(
        HASH2,
        ir::Selector::compute("Hello, World!".as_bytes(), ir::SelectorAbi::Ink).into_be_u32(),
    );

    const HASH3: u32 = ink::selector_id!(Abi::Ink, "message");
    assert_eq!(
        HASH3,
        ir::Selector::compute("message".as_bytes(), ir::SelectorAbi::Ink).into_be_u32(),
    );

    const HASH4: u32 = ink::selector_id!(Abi::Ink, "constructor");
    assert_eq!(
        HASH4,
        ir::Selector::compute("constructor".as_bytes(), ir::SelectorAbi::Ink).into_be_u32(),
    );
}
