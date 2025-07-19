#![allow(unexpected_cfgs)]

// `SolEncode` and `SolDecode` derive macros don't generate implementations for
// enums with fields (see respective docs for rationale).

#[derive(ink::SolDecode, ink::SolEncode)]
enum EnumWithUnnamedFields {
    Unit,
    UnnamedFields(bool, u8, String),
}

#[derive(ink::SolDecode, ink::SolEncode)]
enum EnumWithNamedFields {
    Unit,
    NamedFields {
        status: bool,
        count: u8,
        reason: String,
    },
}

fn main() {}
