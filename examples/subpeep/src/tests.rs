use super::*;

use pdsl_core::memory::vec::Vec;

impl Subpeep {
	/// Returns all recent global posts as vector.
	pub(crate) fn recent_peeps(&self, amount: usize) -> Vec<Peep> {
		self
			.peeps
			.iter()
			.rev()
			.take(amount)
			.cloned()
			.collect()
	}

	/// Returns the `n` most recent peeps of the given user.
	///
	/// Returns `None` if the user does not exist.
	pub(crate) fn recent_user_peeps(
		&self,
		amount: usize,
		username: &str,
	) -> Option<Vec<Peep>> {
		self
			.users
			.get(username)
			.map(|user| {
				user
					.peeps
					.iter()
					.rev()
					.take(amount)
					.cloned()
					.map(|message| {
						Peep::new(username.into(), message)
					})
					.collect()
			})
	}
}

#[macro_use]
use pdsl_core::memory::vec;

#[test]
fn deploy() {
	let subpeep = Subpeep::default();
	assert_eq!(
		subpeep.recent_peeps(10),
		vec![]
	);
	assert_eq!(
		subpeep.recent_user_peeps(10, "alice"),
		None
	);
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
		Some(vec![
			Peep::new(test_user.into(), test_message.into())
		])
	);
}

// #[test]
// fn follow_user() {

// }
