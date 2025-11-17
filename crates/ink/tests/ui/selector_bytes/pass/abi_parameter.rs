use ink_ir as ir;

macro_rules! assert_ink_abi {
    ( $input:literal ) => {{
        const HASH: [u8; 4] = ink::selector_bytes!(Abi::Ink, $input);
        assert_eq!(
            HASH,
            ir::Selector::compute($input.as_bytes(), ir::SelectorAbi::Ink).to_bytes(),
        );
    }};
}

macro_rules! assert_sol_abi {
    ( $input:literal ) => {{
        const HASH: [u8; 4] = ink::selector_bytes!(Abi::Sol, $input);
        assert_eq!(
            HASH,
            ir::Selector::compute($input.as_bytes(), ir::SelectorAbi::Sol).to_bytes(),
        );
    }};
}

fn main() {
    // Test ink! ABI
    assert_ink_abi!("hello");
    assert_ink_abi!("transfer");

    // Test Solidity ABI
    assert_sol_abi!("hello");
    assert_sol_abi!("transfer");

    // Verify they produce different results
    const INK_HELLO: [u8; 4] = ink::selector_bytes!(Abi::Ink, "hello");
    const SOL_HELLO: [u8; 4] = ink::selector_bytes!(Abi::Sol, "hello");
    assert_ne!(INK_HELLO, SOL_HELLO);

    println!("Ink selector for 'hello': {:?}", INK_HELLO);
    println!("Sol selector for 'hello': {:?}", SOL_HELLO);
}
