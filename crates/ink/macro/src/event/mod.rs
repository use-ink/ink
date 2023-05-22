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

mod metadata;

pub use metadata::event_metadata_derive;

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

    // Anonymous events require 1 fewer topics since they do not include their signature.
    let anonymous_topics_offset = usize::from(!anonymous);
    let len_topics = variant.bindings().len() + anonymous_topics_offset;

    let remaining_topics_ty = match len_topics {
        0 => quote_spanned!(span=> ::ink::env::topics::state::NoRemainingTopics),
        n => quote_spanned!(span=> [::ink::env::topics::state::HasRemainingTopics; #n]),
    };

    let event_ident = variant.ast().ident;
    let signature_topic = if !anonymous {
        let topic_bytes = signature_topic(variant.ast().fields, event_ident);
        quote_spanned!(span=> ::core::option::Option::Some([ #( #topic_bytes ),* ]))
    } else {
        quote_spanned!(span=> None)
    };
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

    s.bound_impl(quote!(::ink::env::Topics), quote! {
        type RemainingTopics = #remaining_topics_ty;

        const TOPICS_LEN: usize = #len_topics;
        const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> = #signature_topic;

        fn topics<const MAX_TOPICS: usize, E, B>(
            &self,
            builder: ::ink::env::topics::TopicsBuilder<::ink::env::topics::state::Uninit, E, B>,
        ) -> <B as ::ink::env::topics::TopicsBuilderBackend<E>>::Output
        where
            E: ::ink::env::Environment,
            B: ::ink::env::topics::TopicsBuilderBackend<E>,
        {
            // assert at compile time the number of topics defined by this event is within the
            // given limit.
            let _ = ::ink::codegen::EventRespectsTopicLimit::<{ Self::TOPICS_LEN }, { MAX_TOPICS }>::ASSERT;

            match self {
                #topics_builder
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
