// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_core::{storage, storage::Flush};
use ink_lang::contract;
use scale::{Encode, Decode};

#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

/// Access rights to the shared vector.
#[derive(Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct Access {
    pub begin: Option<u32>,
    pub end: Option<u32>,
}

impl Flush for Access {
	fn flush(&mut self) {}
}

impl Access {
    /// Creates unlimited access rights.
    pub fn unlimited() -> Self {
        Access { begin: None, end: None }
    }

    /// Returns `true` if the access is unlimited.
    pub fn is_unlimited(&self) -> bool {
        self.begin.is_none() && self.end.is_none()
    }

    /// Creates new limited access rights.
    pub fn new<B, E>(begin: B, end: E) -> Self
    where
        B: Into<Option<u32>>,
        E: Into<Option<u32>>,
    {
        let begin = begin.into();
        let end = end.into();
        assert!(begin <= end);
        Access { begin, end }
    }

    /// Returns `true` if the given index is within access rights.
    pub fn contains(&self, index: u32) -> bool {
        let begin = self.begin.unwrap_or(0);
        let end = self.end.unwrap_or(core::u32::MAX);
        begin <= index && index <= end
    }
}

pub type ErrNo = u32;
pub type Result<T, E> = core::result::Result<T, E>;

const ACCESS_NOT_ALLOWED: ErrNo = 1;
const NOT_REGISTERED: ErrNo = 2;
const OUT_OF_BOUNDS: ErrNo = 3;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    /// A shared vector that is accessiable to a subset of allowed mutators.
    struct SharedVec {
        /// The allowed mutators.
        ///
        /// They can operate on a range within the shared vector.
        /// The range is defined by `(start,end)` where `start` and `end`
        /// refer to the zero-starting index. A value of `None` means
        /// that there is no lower or upper bound.
        mutators: storage::HashMap<AccountId, Access>,
        /// The shared vector.
        vec: storage::Vec<i32>,
    }

    /// Fires whenever a new mutator is registered
    /// or when a mutators access rights are changed.
    event Register {
        /// The mutator.
        mutator: AccountId,
        /// The begin access index.
        begin: Option<u32>,
        /// The end access index.
        end: Option<u32>,
    }

    /// Fires whenever a mutator pushes the vector successfully.
    event Push {
        /// The mutator.
        mutator: AccountId,
        /// The pushed value.
        value: i32,
    }

    /// Fires whenever a mutator changes the vector.
    event Mutate {
        /// The index where the change happened.
        at: u32,
        /// The new value.
        value: i32,
    }

    impl Deploy for SharedVec {
        /// Initializes the value to the initial value.
        fn deploy(&mut self) {
            self.mutators.insert(env.caller(), Access::unlimited());
        }
    }

    impl SharedVec {
        /// Returns the users access if registered or an appropriate error.
        fn validate_access(&self, mutator: AccountId) -> Result<&Access, u32> {
            if let Some(access) = self.mutators.get(&mutator) {
                Ok(access)
            } else {
                Err(NOT_REGISTERED)
            }
        }

        /// Pushes a new value to the shared vector.
        ///
        /// # Errors
        ///
        /// - If the caller does not have unlimited access rights.
        pub(external) fn push(&mut self, value: i32) -> Result<(), u32> {
            let access = self.validate_access(env.caller())?;
            if !access.is_unlimited() {
                return Err(ACCESS_NOT_ALLOWED)
            }
            self.vec.push(value);
            env.emit(Push { mutator: env.caller(), value });
            Ok(())
        }

        /// Registers a new user with the given access rights.
        ///
        /// Can also be used to change access rights of an already existing user.
        ///
        /// # Errors
        ///
        /// - If the caller does not have unlimited access rights.
        pub(external) fn register(
            &mut self,
            mutator: AccountId,
            begin: Option<u32>,
            end: Option<u32>
        ) -> Result<(), u32> {
            let access = self.validate_access(env.caller())?;
            if !access.is_unlimited() {
                return Err(ACCESS_NOT_ALLOWED)
            }
            self.mutators.insert(mutator, Access::new(begin, end));
            env.emit(Register { mutator, begin, end });
            Ok(())
        }

        /// Sets the value at the given position to the given value.
        ///
        /// Returns the previous value.
        ///
        /// # Errors
        ///
        /// - If the given position is out of bounds.
        /// - If the caller does not have the required access rights.
        pub(external) fn set(&mut self, at: u32, to: i32) -> Result<i32, u32> {
            let access = self.validate_access(env.caller())?;
            if !access.contains(at) {
                return Err(ACCESS_NOT_ALLOWED)
            }
            let res = self.vec
                .replace(at, move || to)
                .ok_or(OUT_OF_BOUNDS)?;
            env.emit(Mutate { at, value: to });
            Ok(res)
        }

        /// Returns the value of the shared vector at the given position
        /// or `None` if the access is out of bounds.
        pub(external) fn get(&self, at: u32) -> Option<i32> {
            self.vec.get(at).cloned()
        }

        /// Returns the length of the shared vector.
        pub(external) fn len(&self) -> u32 {
            self.vec.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ink_core::env;
    use std::convert::TryFrom;
    use ink_core::env::DefaultSrmlTypes;

    #[test]
    fn it_works() {
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let bob = AccountId::try_from([0x1; 32]).unwrap();
        let charly = AccountId::try_from([0x2; 32]).unwrap();

        env::test::set_caller::<DefaultSrmlTypes>(alice);

        let mut shared_vec = SharedVec::deploy_mock();
        assert_eq!(shared_vec.len(), 0);
        assert_eq!(shared_vec.push(5), Ok(()));
        assert_eq!(shared_vec.len(), 1);
        assert_eq!(shared_vec.register(bob, Some(0), Some(1)), Ok(()));
        assert_eq!(shared_vec.push(42), Ok(()));
        assert_eq!(shared_vec.push(1337), Ok(()));
        assert_eq!(shared_vec.push(77), Ok(()));
        assert_eq!(shared_vec.len(), 4);
        assert_eq!(shared_vec.set(1, 1000), Ok(42));
        assert_eq!(shared_vec.set(2, 2000), Ok(1337));
        assert_eq!(shared_vec.set(5, 3000), Err(OUT_OF_BOUNDS));
        assert_eq!(shared_vec.get(0), Some(5));
        assert_eq!(shared_vec.get(1), Some(1000));
        assert_eq!(shared_vec.get(2), Some(2000));
        assert_eq!(shared_vec.get(3), Some(77));

        env::test::set_caller::<DefaultSrmlTypes>(bob);

        assert_eq!(shared_vec.set(1, 999), Ok(1000));
        assert_eq!(shared_vec.set(2, 1999), Err(ACCESS_NOT_ALLOWED));
        assert_eq!(shared_vec.set(5, 3000), Err(ACCESS_NOT_ALLOWED));
        assert_eq!(shared_vec.register(charly, Some(0), Some(2)), Err(ACCESS_NOT_ALLOWED));
        assert_eq!(shared_vec.get(0), Some(5));
        assert_eq!(shared_vec.get(1), Some(999));
        assert_eq!(shared_vec.get(2), Some(2000));
        assert_eq!(shared_vec.get(3), Some(77));

        env::test::set_caller::<DefaultSrmlTypes>(charly);

        assert_eq!(shared_vec.set(1, 888), Err(NOT_REGISTERED));
        assert_eq!(shared_vec.set(5, 3000), Err(NOT_REGISTERED));
        assert_eq!(shared_vec.register(charly, Some(1), Some(1)), Err(NOT_REGISTERED));
        assert_eq!(shared_vec.get(0), Some(5));
        assert_eq!(shared_vec.get(1), Some(999));
        assert_eq!(shared_vec.get(2), Some(2000));
        assert_eq!(shared_vec.get(3), Some(77));
    }
}
