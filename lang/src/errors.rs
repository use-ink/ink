use proc_macro2::TokenStream;
use quote::ToTokens;
use std::result::Result as StdResult;
pub use syn::parse::Error as SynError;

macro_rules! bail {
    ($($args:tt)*) => {
        return Err(format_err!($($args)*).into())
    }
}

macro_rules! format_err {
    ($tokens:expr, $($msg:tt)*) => {
        match &$tokens {
            t => {
                syn::parse::Error::new_spanned(t, format_args!($($msg)*))
            }
        }
    }
}

/// A collection of errors.
///
/// # Note
///
/// This is used to allow for reporting multiple errors at the same time.
#[derive(Debug)]
pub struct Errors {
	errors: Vec<SynError>,
}

impl From<SynError> for Errors {
    fn from(err: SynError) -> Errors {
        Errors{
			errors: vec![err],
		}
    }
}

impl From<Vec<Errors>> for Errors {
    fn from(err: Vec<Errors>) -> Errors {
        let result = err
			.into_iter()
			.flat_map(|v| v.errors)
			.collect::<Vec<_>>();
        assert!(result.len() > 0);
        Errors{ errors: result }
    }
}

/// Used to create a TokenStream from a list of errors
impl ToTokens for Errors {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for item in self.errors.iter() {
            item.to_compile_error().to_tokens(tokens);
        }
    }
}

/// Result type alias for an error type which allows for accumulating errors.
pub type Result<T> = StdResult<T, Errors>;
