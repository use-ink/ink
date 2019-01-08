use super::*;

#[test]
fn deploy() {
	let subpeep = Subpeep::default();
	assert_eq!(
		subpeep.recent_tweets(10),
		vec![]
	);
	assert_eq!(
		subpeep.recent_user_tweets(10, "alice"),
		None
	);
}

#[test]
fn tweet_message() {
	let mut subpeep = Subpeep::default();
	let test_user = "Alice";
	let test_message = "Hello, World!";
	subpeep.register(test_user.into());
	subpeep.tweet_message(test_user.into(), test_message.into());
	assert_eq!(
		subpeep.recent_tweets(10),
		vec![Tweet::new(test_user.into(), test_message.into())],
	);
	assert_eq!(
		subpeep.recent_user_tweets(10, test_user.into()),
		Some(vec![
			Tweet::new(test_user.into(), test_message.into())
		])
	);
}

// #[test]
// fn follow_user() {

// }
