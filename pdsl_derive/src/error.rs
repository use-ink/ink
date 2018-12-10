pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	ModWithoutBody{
		/// The name of the module
		name: syn::Ident
	},
	InvalidModVisibility{
		/// The name of the module
		name: syn::Ident,
		/// The invalid visibility of the module
		invalid_vis: syn::Visibility
	}
}

impl Error {
	pub fn mod_without_body(name: syn::Ident) -> Self {
		Error::ModWithoutBody{name}
	}

	pub fn invalid_mod_visibility(name: syn::Ident, invalid_vis: syn::Visibility) -> Self {
		Error::InvalidModVisibility{name, invalid_vis}
	}
}

impl std::error::Error for Error {
	fn description(&self) -> &str {
		match self {
			Error::ModWithoutBody{..} => {
				"encountered unbraced module definition, \
				 use `mod foo {{ ... }}` instead of `mod foo;`"
			}
			Error::InvalidModVisibility{..} => {
				"encountered invalid module visibility modifier, \
				 use `pub` modifier, like in `pub mod foo { ... }`"
			}
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Error::ModWithoutBody{name} => {
				std::writeln!(
					f,
					"[substrate_contract] error: \
					module '{}' needs to be braced, e.g. `mod foo {{ ... }}`",
					name.to_string()
				)
			}
			Error::InvalidModVisibility{name,invalid_vis} => {
				let vis_str = match invalid_vis {
					syn::Visibility::Public(_) => Some("pub"),
					syn::Visibility::Crate(_) => Some("crate"),
					syn::Visibility::Restricted(_) => Some("pub(restricted)"),
					syn::Visibility::Inherited => None
				};
				if let Some(vis) = vis_str {
					std::writeln!(
						f,
						"[substrate_contract] error: \
						module '{}' requires public visibility but has visibility `{}` instead",
						name.to_string(),
						vis
					)
				} else {
					std::writeln!(
						f,
						"[substrate_contract] error: \
						module '{}' requires public visibility but has no visibility specified",
						name.to_string()
					)
				}
			}
		}
	}
}
