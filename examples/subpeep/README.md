# Subpeep

Decentralized message distribution inspired by Twitter.

## Features

### Required

- Register users by their name.
- Checks if the account is allowed to tweet.
- Users can tweet messages that are prodcasted to the global channel.
- Users can follow other users by their name.

## Data Structures

```rust
// All global tweets.
GLOBAL_TWEETS = [Tweet; 10]

// The address for the registered user
AUTH = mapping Username -> Address
// All tweets by a single user
USER_TWEETS = mapping Username -> Vec<Tweet>
// All users that this user is following
USER_FOLLOWS = mapping Username -> Vec<Username>
```

## API

**Note:** `caller()` returns the senders address.

```python
fn register(username)
    AUTH[username] = caller()

fn tweet(username, tweet)
    if AUTH[username] = caller()
        TWEETS[username].append(tweet)

fn follow(username: String, followed: String)
    if AUTH[username] == caller() and AUTH[followed].is_some()
        FOLLOWING[username].append(followed)
```
