use ink_lang as ink;
use ink_lang_ir as ir;

macro_rules! assert_macro_eq {
    ( $input:literal ) => {{
        // We put it into a constant to verify that the computation is constant.
        const HASH: [u8; 32] = ink::blake2x256!($input);
        assert_eq!(
            HASH,
            {
                let mut output = [0u8; 32];
                ir::blake2b_256($input.as_bytes(), &mut output);
                output
            }
        );
    }};
}

fn main() {
    assert_macro_eq!("");
    assert_macro_eq!("Hello, World!");
    assert_macro_eq!("message");
    assert_macro_eq!("constructor");
}
