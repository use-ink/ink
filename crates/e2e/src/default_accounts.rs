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

//! Default accounts provided by [`sp_keyring::sr25519::Keyring`].

use super::{
    AccountKeyring,
    PairSigner,
};
use sp_core::sr25519;

#[rustfmt::skip]
macro_rules! default_account {
    ($fn_name:ident $keyring_fn_name:ident) => {
        #[doc = concat!(
            "Returns the default [`sp_keyring::sr25519::Keyring`] for `//",
            stringify!($keyring_fn_name),
            "`."
        )]
        pub fn $fn_name<C>() -> PairSigner<C, sr25519::Pair>
        where
            C: subxt::Config,
            C::Signature: From<sr25519::Signature>,
            C::AccountId: From<sp_core::crypto::AccountId32>
        {
            PairSigner::new(AccountKeyring::$keyring_fn_name.pair())
        }
    };
}

// The following accounts are pre-defined in [`sp-keyring::sr25519`].
default_account!(alice Alice);
default_account!(bob Bob);
default_account!(charlie Charlie);
default_account!(dave Dave);
default_account!(eve Eve);
default_account!(ferdie Ferdie);
default_account!(one One);
default_account!(two Two);
