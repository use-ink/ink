use syn;

/// An item definition marked with `#[contract]`.
///
/// This can be either a `struct` or `enum` definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractItem {
	Struct(ContractStruct),
	Enum(ContractEnum),
}

/// A `struct` definition that was marked as contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractStruct {
	pub item: syn::ItemStruct
}

/// An `enum` definition that was marked as contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractEnum {
	pub item: syn::ItemEnum
}

/// Module definitions with bodies, e.g. `mod foo { ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractModule {
	pub vis: syn::VisPublic,
	pub mod_tok: syn::token::Mod,
	pub ident: syn::Ident,
    pub attrs: Vec<syn::Attribute>,
	pub brace: syn::token::Brace,
    pub items: Vec<syn::Item>,
	pub contracts: Vec<ContractItem>,
}
