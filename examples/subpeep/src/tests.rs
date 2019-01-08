use super::*;

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
