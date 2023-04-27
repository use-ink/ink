// Copyright 2018-2022 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned;

/// Derives the `ink::Event` trait for the given `struct`.
pub fn event_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::Move)
        .add_bounds(synstructure::AddBounds::Fields)
        .underscore_const(true);
    match &s.ast().data {
        syn::Data::Struct(_) => event_derive_struct(s),
        _ => {
            panic!("can only derive `Event` for Rust `struct` items")
        }
    }
}

/// `Event` derive implementation for `struct` types.
fn event_derive_struct(mut s: synstructure::Structure) -> TokenStream2 {
    assert_eq!(s.variants().len(), 1, "can only operate on structs");
    let span = s.ast().span();

    let anonymous = false; // todo read from struct attribute e.g. #[event(anonymous)]

    // let decode_body = variant.construct(|field, _index| {
    //     let ty = &field.ty;
    //     let span = ty.span();
    //     quote_spanned!(span =>
    //         <#ty as ::ink::storage::traits::Storable>::decode(__input)?
    //     )
    // });
    // let encode_body = variant.each(|binding| {
    //     let span = binding.ast().ty.span();
    //     quote_spanned!(span =>
    //         ::ink::storage::traits::Storable::encode(#binding, __dest);
    //     )
    // });
    s.variants_mut()[0].filter(|bi| {
        bi.ast()
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("topic"))
    });

    let variant = &s.variants()[0];

    let len_topics = variant.bindings().len();
    // Anonymous events require 1 fewer topics since they do not include their signature.
    let anonymous_topics_offset = usize::from(!anonymous);
    let remaining_topics_ty = match len_topics + anonymous_topics_offset {
        0 => quote_spanned!(span=> ::ink::env::topics::state::NoRemainingTopics),
        n => quote_spanned!(span=> [::ink::env::topics::state::HasRemainingTopics; #n]),
    };

    let signature_topic = signature_topic(variant.ast().fields, &variant.ast().ident);
    let event_signature_topic = if anonymous {
        None
    } else {
        Some(quote_spanned!(span=>
            .push_topic(&Self::SIGNATURE_TOPIC.expect("non-anonymous events must have a signature topic"))
        ))
    };

    let pat = variant.pat();
    let topics = variant.bindings().iter().fold(quote!(), |acc, field| {
        let field_ty = &field.ast().ty;
        let field_span = field_ty.span();
        quote_spanned!(field_span=>
            #acc
            .push_topic::<#field_ty>(#field)
        )
    });
    let topics_builder = quote!(
        #pat => {
            builder
                .build::<Self>()
                #event_signature_topic
                #topics
                .finish()
        }
    );

    s.gen_impl(quote! {
         gen impl ::ink::env::Topics for @Self {
            type RemainingTopics = #remaining_topics_ty;

            fn topics<E, B>(
                &self,
                builder: ::ink::env::topics::TopicsBuilder<::ink::env::topics::state::Uninit, E, B>,
            ) -> <B as ::ink::env::topics::TopicsBuilderBackend<E>>::Output
            where
                E: ::ink::env::Environment,
                B: ::ink::env::topics::TopicsBuilderBackend<E>,
            {
                match self {
                    #topics_builder
                }
            }
         }
     })
}

/// The signature topic of an event variant.
///
/// Calculated with `blake2b("Event(field1_type,field2_type)")`.
pub fn signature_topic(
    fields: &syn::Fields,
    event_ident: &syn::Ident,
) -> [syn::LitInt; 32] {
    let fields = fields
        .iter()
        .map(|field| {
            quote::ToTokens::to_token_stream(&field.ty)
                .to_string()
                .replace(" ", "")
        })
        .collect::<Vec<_>>()
        .join(",");
    let topic_str = format!("{}({fields})", event_ident);
    let input = topic_str.as_bytes();
    let mut output = [0; 32];
    ink_ir::blake2b_256(&input, &mut output);
    output.map(<u8 as ink_ir::HexLiteral>::hex_padded_suffixed)
}
