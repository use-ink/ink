use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
    Span,
};
use syn::{
    Token,
    punctuated::Punctuated,
    spanned::Spanned as _,
};
use quote::quote;
use derive_more::From;

/// The contract with all required information.
pub struct Contract {
    /// The `mod` token.
    pub mod_token: Token![mod],
    /// The modules snake case identifier.
    pub ident: Ident,
    /// Special ink! meta attributes.
    pub meta_items: Vec<ItemMeta>,
    /// Outer and inner attributes of the module.
    ///
    /// This also containes the environmental types definition
    /// as well as the ink! version.
    pub attrs: Vec<syn::Attribute>,
    /// The state struct.
    pub storage: ItemStorage,
    /// All event structs.
    pub events: Vec<ItemEvent>,
    /// Messages, constructors and methods of the contract.
    pub functions: Vec<Function>,
}

/// Types implementing this trait are code generators for the ink! language.
pub trait GenerateCode {
    /// Generates ink! contract code.
    fn generate_code(&self) -> TokenStream2;
}

impl GenerateCode for Contract {
    /// Generates ink! contract code.
    fn generate_code(&self) -> TokenStream2 {
        quote! {}
    }
}

/// Intermediate description of a contracts components.
#[derive(From)]
pub enum Item {
    /// A meta item.
    Meta(ItemMeta),
    /// A storage definition.
    Storage(ItemStorage),
    /// An event definition.
    Event(ItemEvent),
    /// An implementation block.
    Impl(ItemImpl),
}

/// A specialized ink! meta attribute.
///
/// # Note
///
/// It is possible to set multiple meta items with one meta attribute:
///
/// ```no_compile
/// #[ink(
///     env = DefaultSrmlTypes,
///     version = 0.1.0,
/// )]
/// ```
#[derive(Debug, From)]
pub enum ItemMeta {
    /// Environmental types definition: `#[ink(env = DefaultSrmlTypes)]`
    Env(MetaEnv),
    /// Information about the ink! version: `#[ink(version = X.Y.Z)]`
    Version(MetaVersion),
    /// Simple single-identifier ink! attribute: e.g. `#[ink(storage)]` or `#[ink(event)]`
    Simple(MetaSimple),
}

/// Simple single-identifier ink! attribute: e.g. `#[ink(storage)]` or `#[ink(event)]`
#[derive(Debug)]
pub struct MetaSimple {
    /// The parentheses around the `event` identifier.
    pub paren_token: syn::token::Paren,
    /// The simple identifier.
    pub ident: Ident,
}

impl MetaSimple {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.paren_token.span
    }
}

impl PartialEq<str> for MetaSimple {
    fn eq(&self, other: &str) -> bool {
        self.ident == other
    }
}

impl ItemMeta {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        match self {
            ItemMeta::Env(meta_env) => meta_env.span(),
            ItemMeta::Version(meta_version) => meta_version.span(),
            ItemMeta::Simple(meta_simple) => meta_simple.span(),
        }
    }

    /// Returns the ink! attribute if it is simple and the idents match or `None`.
    ///
    /// # Examples
    ///
    /// Simple attributes are for example `#[ink(storage)]` where `storage` is its ident.
    fn filter_simple_by_ident(&self, ident: &str) -> Option<&MetaSimple> {
        match self {
            ItemMeta::Simple(meta_simple) if meta_simple == ident => Some(meta_simple),
            _ => None,
        }
    }

    /// Returns `true` if the ink! attribute is simple and equal to the given ident.
    ///
    /// # Examples
    ///
    /// Simple attributes are for example `#[ink(storage)]` where `storage` is its ident.
    pub fn is_simple(&self, ident: &str) -> bool {
        self.filter_simple_by_ident(ident).is_some()
    }

    /// Returns the simple ink! attribute identifier and `None` if `self` is not simple.
    pub fn get_simple(&self) -> Option<&Ident> {
        match self {
            ItemMeta::Simple(meta_simple) => Some(&meta_simple.ident),
            _ => None,
        }
    }
}

/// The environment types definition: `#[ink(env = DefaultSrmlTypes)]`
#[derive(Debug)]
pub struct MetaEnv {
    /// The `env` identifier.
    env: Ident,
    /// The `=` token.
    eq_token: Token![=],
    /// The environmental types type.
    ty: syn::Type,
}

impl MetaEnv {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.env
            .span()
            .join(self.ty.span())
            .expect("both spans are in the same file AND we are using nightly Rust; qed")
    }
}

/// The used ink! version: `#[ink(version = 0.1.0)]`
#[derive(Debug)]
pub struct MetaVersion {
    /// The `version` identifier.
    version: Ident,
    /// The `=` token.
    eq_token: Token![=],
    /// The major version.
    major: syn::LitInt,
    /// The first dot `.`.
    dot_1: Token![.],
    /// The minor version.
    minor: syn::LitInt,
    /// The second dot `.`.
    dot_2: Token![.],
    /// The patch version.
    patch: syn::LitInt,
}

impl MetaVersion {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.version
            .span()
            .join(self.patch.span())
            .expect("both spans are in the same file AND we are using nightly Rust; qed")
    }
}

/// The state struct of the contract.
pub struct ItemStorage {
    /// The `struct` token.
    pub struct_token: Token![struct],
    /// The storage name.
    ///
    /// This normally is equal to the contract's name but camel case.
    pub ident: Ident,
    /// Outer attributes of the storage struct.
    pub attrs: Vec<syn::Attribute>,
    /// Fields of the storage struct.
    pub fields: syn::FieldsNamed,
}

/// An event struct.
pub struct ItemEvent {
    /// The `struct` token.
    pub struct_token: Token![struct],
    /// The name of the event.
    pub ident: Ident,
    /// Outer attributes of the event struct.
    pub attrs: Vec<syn::Attribute>,
    /// Fields of the event struct.
    pub fields: syn::FieldsNamed,
}

/// An implementation block in ink!.
pub struct ItemImpl {
    /// Inner attributes.
    pub attrs: Vec<syn::Attribute>,
    /// The `impl` token.
    pub impl_token: Token![impl],
    /// The implementer type.
    pub self_ty: Ident,
    /// The `{` and `}` tokens.
    pub brace_token: syn::token::Brace,
    /// The functions.
    pub functions: Vec<Function>,
}

/// Represents an ink! message, an ink! constructor or a normal method.
pub struct Function {
    /// The attributes of the function.
    pub attrs: Vec<syn::Attribute>,
    /// The kind of the function.
    pub kind: FunctionKind,
    /// The signature of the function.
    pub sig: Signature,
    /// The statements of the function.
    pub block: syn::Block,
}

/// The kind of a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKind {
    /// A contract constructor.
    Constructor,
    /// A contract message.
    Message,
    /// A normal (private) method.
    Method,
}

impl Function {
    /// Returns `true` if the function is a contract constructor.
    pub fn is_constructor(&self) -> bool {
        self.kind == FunctionKind::Constructor
    }

    /// Returns `true` if the function is a contract message.
    pub fn is_message(&self) -> bool {
        self.kind == FunctionKind::Message
    }

    /// Returns `true` if the function is a method.
    pub fn is_method(&self) -> bool {
        self.kind == FunctionKind::Method
    }
}

/// The signature of an ink! message, an ink! constructor or a normal method.
pub struct Signature {
    /// The `fn` token.
    pub fn_token: Token![fn],
    /// The identifier.
    pub ident: Ident,
    /// The functions generics.
    ///
    /// Only applicable to methods.
    pub generics: syn::Generics,
    /// The parentheses `(` and `)`.
    pub paren_token: syn::token::Paren,
    /// The function inputs, delimited by `,`.
    pub inputs: Punctuated<FnArg, Token![,]>,
    /// The function output.
    pub output: syn::ReturnType,
}

impl Signature {
    /// Returns `true` if the signature is `&mut self`.
    pub fn is_mut(&self) -> bool {
        self.self_arg().mutability.is_some()
    }

    /// Returns the `self` input.
    pub fn self_arg(&self) -> &syn::Receiver {
        if let FnArg::Receiver(receiver) = &self.inputs[0] {
            return &receiver
        } else {
            unreachable!("must contain the receiver in the first argument position")
        }
    }

    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.fn_token.span().join(self.output.span())
            .expect("spans of `ident` and `ty` must be in the same file; qed")
    }
}

/// A single input of a function.
pub enum FnArg {
    /// Either `&self` or `&mut self`.
    ///
    /// Note that `syn::Receiver` might also represent `self`.
    Receiver(syn::Receiver),
    /// A captured arguments: e.g. `a: i32`
    Typed(IdentType),
}

/// A captured argument.
///
/// # Examples
///
/// - `a: i32`
/// - `foo: Vec<u8>`
pub struct IdentType {
    /// The attributes of the argument.
    pub attrs: Vec<syn::Attribute>,
    /// The identifier.
    pub ident: Ident,
    /// The `:` token.
    pub colon_token: Token![:],
    /// The type.
    pub ty: syn::Type,
}

impl IdentType {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.ident.span().join(self.ty.span())
            .expect("spans of `ident` and `ty` must be in the same file; qed")
    }
}
