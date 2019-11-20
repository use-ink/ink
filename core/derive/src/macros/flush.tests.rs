
pub trait Flush {
    fn flush(&mut self);
}

impl Flush for bool { fn flush(&mut self) {} }
impl Flush for u8 { fn flush(&mut self) {} }
impl Flush for u16 { fn flush(&mut self) {} }
impl Flush for u32 { fn flush(&mut self) {} }
impl Flush for i8 { fn flush(&mut self) {} }
impl Flush for i16 { fn flush(&mut self) {} }
impl Flush for i32 { fn flush(&mut self) {} }
impl Flush for String { fn flush(&mut self) {} }
impl Flush for Vec<u8> { fn flush(&mut self) {} }
impl Flush for [u8; 32] { fn flush(&mut self) {} }

enum Empty {}

impl Flush for Empty {
    fn flush(&mut self) {}
}

enum CStyle {
    A, B, C,
}

impl Flush for CStyle {
    fn flush(&mut self) {
        match self {
            CStyle::A => (),
            CStyle::B => (),
            CStyle::C => (),
        }
    }
}

enum TupleStruct {
    A(bool),
    B(i8, i16),
    C(String, Vec<u8>, [u8; 32]),
}

impl Flush for TupleStruct {
    fn flush(&mut self) {
        match self {
            Self::A(_0) => {
                Flush::flush(_0);
            },
            Self::B(_0, _1) => {
                Flush::flush(_0);
                Flush::flush(_1);
            },
            Self::C(_0, _1, _2) => {
                Flush::flush(_0);
                Flush::flush(_1);
                Flush::flush(_2);
            },
        }
    }
}

enum Struct {
    A { a: bool },
    B { a: i8, b: i16 },
    C { a: String, b: Vec<u8>, c: [u8; 32] },
}

impl Flush for Struct {
    fn flush(&mut self) {
        match self {
            Self::A { a } => {
                Flush::flush(a);
            },
            Self::B { a, b } => {
                Flush::flush(a);
                Flush::flush(b);
            },
            Self::C { a, b, c } => {
                Flush::flush(a);
                Flush::flush(b);
                Flush::flush(c);
            },
        }
    }
}

enum Mixed {
    A,
    B(String, Vec<u8>, [u8; 32]),
    C { a: String, b: Vec<u8>, c: [u8; 32] },
}

impl Flush for Mixed {
    fn flush(&mut self) {
        match self {
            Self::A => (),
            Self::B(a, b, c) => {
                Flush::flush(a);
                Flush::flush(b);
                Flush::flush(c);
            },
            Self::C { a, b, c } => {
                Flush::flush(a);
                Flush::flush(b);
                Flush::flush(c);
            },
        }
    }
}
