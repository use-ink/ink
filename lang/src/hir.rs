use crate::{
    ast,
    errors::{Result, SynError}
};
use syn::token;
use proc_macro2::{Ident, Span};

/// A smart contract.
#[derive(Debug)]
pub struct Contract {
    /// The name of the smart contract.
    name: Ident,
	/// The storage state fields.
	state: State,
    /// The deploy handler of the smart contract.
    on_deploy: OnDeployHandler,
    /// The messages of the smart contract.
    messages: Vec<Message>,
    /// Methods of the smart contract.
    methods: Vec<Method>,
}

impl Contract {
	/// Extracts the contract state from the contract items
	/// and performs some integrity checks on it.
	fn extract_state(contract: &ast::Contract) -> Result<(&Ident, State)> {
        let states = contract.states().collect::<Vec<_>>();
        if states.is_empty() {
			return Err(
				SynError::new(
					Span::call_site(),
					"requires exactly one contract state `struct`; found none"
				).into()
			)
        }
        if states.len() > 1 {
			return Err(
				SynError::new(
					Span::call_site(),
					format!(
						"requires exactly one contract state `struct`; found {:?}",
						states.len()
					)
				).into()
			)
        }
		let state = states[0];
		Ok((&state.ident, State::from(state)))
	}

	fn unpack_impl_blocks(contract_ident: &Ident, contract: &ast::Contract)
		-> Result<(OnDeployHandler, Vec<Message>, Vec<Method>)>
	{
        let impl_blocks = contract.impl_blocks().collect::<Vec<_>>();
        if impl_blocks.is_empty() {
			return Err(
				SynError::new(
					Span::call_site(),
					"requires at least one contract impl block `struct`; found none"
				).into()
			)
        }
		for impl_block in impl_blocks.iter() {
			if impl_block.self_ty != *contract_ident {
				bail!(
					impl_block.self_ty,
					"contract impl blocks must implement for the contract type: {}",
					contract_ident
				)
			}
		}
		use itertools::Itertools as _;
		let (mut messages, methods): (Vec<_>, Vec<_>) = impl_blocks
			.into_iter()
			.flat_map(|impl_block| impl_block.items.iter())
			.partition_map(|msg_or_method| {
				use either::Either;
				if msg_or_method.vis.is_external() {
					Either::Left(Message::from(msg_or_method))
				} else {
					Either::Right(Method::from(msg_or_method))
				}
			});
		let deploy_handler_idx = messages
			.iter()
			.position(|msg| msg.sig.ident == "on_deploy");
		let deploy_handler = match deploy_handler_idx {
			Some(idx) => {
				messages.swap_remove(idx).into()
			}
			None => {
				return Err(
					SynError::new(
						Span::call_site(),
						"could not find contract deploy handler: `on_deploy`"
					).into()
				)
			}
		};
		for msg in messages.iter() {
			let inputs = &msg.sig.decl.inputs;
			{
				let self_ty: &ast::FnArg = inputs.first().unwrap().into_value();
				match self_ty {
					ast::FnArg::SelfValue(_) | ast::FnArg::Captured(_) => {
						bail!(
							self_ty,
							"contract messages must start with `&self` or `&mut self`"
						)
					}
					_ => ()
				}
			}
		}
		Ok((deploy_handler, messages, methods))
	}

    pub fn from_ast(contract: &ast::Contract) -> Result<Self> {
        let (ident, state) = Self::extract_state(contract)?;
		let (deploy_handler, messages, methods) = Self::unpack_impl_blocks(ident, contract)?;
		Ok(
			Self {
				name: ident.clone(),
				state,
				on_deploy: deploy_handler,
				messages,
				methods,
			}
		)
    }
}

#[derive(Debug)]
pub struct State {
    /// The attributes.
    ///
    /// # Note
    ///
    /// Also used for documentation.
	pub attrs: Vec<syn::Attribute>,
	/// The state fields.
	///
	/// # Note
	///
	/// These are the fields that are going to
	/// be stored in the contract storage.
	pub fields: syn::FieldsNamed,
}

impl From<&ast::ItemState> for State {
	fn from(state: &ast::ItemState) -> Self {
		Self {
			attrs: state.attrs.clone(),
			fields: state.fields.clone(),
		}
	}
}

/// The deploy handler of a smart contract.
///
/// # Note
///
/// This is what is getting called upon deploying a smart contract.
/// Normally this is used to initialize storage values.
#[derive(Debug)]
pub struct OnDeployHandler {
    /// The attributes.
    ///
    /// # Note
    ///
    /// Also used for documentation.
    pub attrs: Vec<syn::Attribute>,
    /// The function declaration.
    pub decl: ast::FnDecl,
    /// The actual implementation.
    pub block: syn::Block,
}

impl From<Message> for OnDeployHandler {
	fn from(msg: Message) -> Self {
		Self {
			attrs: msg.attrs,
			decl: msg.sig.decl,
			block: msg.block,
		}
	}
}

/// A message that is handled by the smart contract.
///
/// # Note
///
/// Messages of a smart contract are only callable externally.
/// They are used to communicate with other smart contracts.
#[derive(Debug)]
pub struct Message {
    /// The attributes.
    ///
    /// # Note
    ///
    /// Also used for documentation.
    pub attrs: Vec<syn::Attribute>,
    /// The message signature.
    ///
    /// # Note
    ///
    /// This also holds the name of the message.
    pub sig: ast::MethodSig,
    /// The actual implementation.
    pub block: syn::Block,
}

impl From<&ast::ItemImplMethod> for Message {
	fn from(impl_method: &ast::ItemImplMethod) -> Self {
		Self {
			attrs: impl_method.attrs.clone(),
			sig: impl_method.sig.clone(),
			block: impl_method.block.clone(),
		}
	}
}

/// A method defined on the smart contract.
#[derive(Debug)]
pub struct Method {
    /// The attributes.
    ///
    /// # Note
    ///
    /// Also used for documentation.
    pub attrs: Vec<syn::Attribute>,
    /// The method visibility.
    ///
    /// # Note
    ///
    /// Currently only inherent visibility (private) is
    /// available for methods.
    pub vis: ast::MethodVisibility,
    /// The method signature.
    ///
    /// # Note
    ///
    /// This also holds the name of the method.
    pub sig: ast::MethodSig,
    /// The actual implementation.
    pub block: syn::Block,
}

impl From<&ast::ItemImplMethod> for Method {
	fn from(impl_method: &ast::ItemImplMethod) -> Self {
		Self {
			attrs: impl_method.attrs.clone(),
			sig: impl_method.sig.clone(),
			vis: impl_method.vis.clone(),
			block: impl_method.block.clone(),
		}
	}
}
