use ink_lang as ink;
use ink_lang_ir as ir;

macro_rules! assert_macro_eq {
    ( $input:literal ) => {{
        // We put it into a constant to verify that the computation is constant.
        const HASH: [u8; 4] = ink::selector_bytes!($input);
        assert_eq!(
            HASH,
            *ir::Selector::new($input.as_bytes()).as_bytes(),
        );
    }};
}

fn main() {
    assert_macro_eq!("");
    assert_macro_eq!("Hello, World!");
    assert_macro_eq!("message");
    assert_macro_eq!("constructor");
}
