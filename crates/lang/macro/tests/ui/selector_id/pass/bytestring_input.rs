use ink_lang as ink;
use ink_lang_ir as ir;

macro_rules! assert_macro_eq {
    ( $input:literal ) => {{
        // We put it into a constant to verify that the computation is constant.
        const HASH: u32 = ink::selector_id!($input);
        assert_eq!(
            HASH,
            ir::Selector::compute($input).into_be_u32(),
        );
    }};
}

fn main() {
    assert_macro_eq!(b"");
    assert_macro_eq!(b"Hello, World!");
    assert_macro_eq!(b"message");
    assert_macro_eq!(b"constructor");
}
