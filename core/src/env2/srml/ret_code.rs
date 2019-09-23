use derive_more::From;

/// A return code which is the result of an external SRML call.
#[derive(Debug, Copy, Clone, PartialEq, Eq, From)]
pub struct RetCode {
    code: u32,
}

impl RetCode {
    /// Creates a `success` indicating return code.
    pub fn success() -> Self {
        Self { code: 0 }
    }

    /// Returns the `u32` representation of `self`.
    pub fn to_u32(self) -> u32 {
        self.code
    }
}
