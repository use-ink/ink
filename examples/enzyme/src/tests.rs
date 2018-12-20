use super::*;

#[test]
fn deploy() {
	let mut enzyme = Enzyme::default();
	enzyme.setup();
	assert_eq!(
		enzyme.recent_tweets(10),
		vec![]
	);
	assert_eq!(
		enzyme.recent_user_tweets(10, "alice"),
		None
	);
}

#[test]
fn tweet_message() {
	let mut enzyme = Enzyme::default();
	enzyme.setup();
	let test_user = "Alice";
	let test_message = "Hello, World!";
	enzyme.register(test_user.into());
	enzyme.tweet_message(test_user.into(), test_message.into());
	assert_eq!(
		enzyme.recent_tweets(10),
		vec![Tweet::new(test_user.into(), test_message.into())],
	);
	assert_eq!(
		enzyme.recent_user_tweets(10, test_user.into()),
		Some(vec![
			Tweet::new(test_user.into(), test_message.into())
		])
	);
}

// #[test]
// fn follow_user() {

// }
