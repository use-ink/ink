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

use super::*;

use ink_core::memory::vec::Vec;

impl Subpeep {
    /// Returns all recent global posts as vector.
    pub(crate) fn recent_peeps(&self, amount: usize) -> Vec<Peep> {
        self.peeps.iter().rev().take(amount).cloned().collect()
    }

    /// Returns the `n` most recent peeps of the given user.
    ///
    /// Returns `None` if the user does not exist.
    pub(crate) fn recent_user_peeps(
        &self,
        amount: usize,
        username: &str,
    ) -> Option<Vec<Peep>> {
        self.users.get(username).map(|user| {
            user.peeps
                .iter()
                .rev()
                .take(amount)
                .cloned()
                .map(|message| Peep::new(username.into(), message))
                .collect()
        })
    }
}

#[macro_use]
use ink_core::memory::vec;

#[test]
fn deploy() {
    let subpeep = Subpeep::default();
    assert_eq!(subpeep.recent_peeps(10), Vec::new());
    assert_eq!(subpeep.recent_user_peeps(10, "alice"), None);
}

#[test]
fn peep_message() {
    let mut subpeep = Subpeep::default();
    let test_user = "Alice";
    let test_message = "Hello, World!";
    subpeep.register(test_user.into());
    subpeep.peep_message(test_user.into(), test_message.into());
    assert_eq!(
        subpeep.recent_peeps(10),
        vec![Peep::new(test_user.into(), test_message.into())],
    );
    assert_eq!(
        subpeep.recent_user_peeps(10, test_user.into()),
        Some(vec![Peep::new(test_user.into(), test_message.into())])
    );
}

// #[test]
// fn follow_user() {

// }
