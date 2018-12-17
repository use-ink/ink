use super::*;

#[test]
fn deploy() {
	let mut enzyme = Enzyme::default();
	enzyme.setup();
	assert_eq!(
		enzyme.global_tweets(),
		vec![String::new(); 10]
	);
	assert_eq!(
		enzyme.recent_user_tweets(10, String::from("alice")),
		Vec::<Tweet>::new()
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
		enzyme.global_tweets(),
		{
			let mut vec = Vec::new();
			vec.push(test_message.clone());
			for _ in 0..(10-1) {
				vec.push("".into());
			}
			vec
		}
	);
	assert_eq!(
		enzyme.recent_user_tweets(10, test_user.into()),
		vec![
			test_message.clone()
		]
	);
}
