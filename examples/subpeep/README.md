# Subpeep

Decentralized message distribution inspired by Twitter.

## Features

### Required

- Register users by their name.
- Checks if the account is allowed to peep.
- Users can peep messages that are prodcasted to the global channel.
- Users can follow other users by their name.

## Data Structures

```rust
// All global peeps.
GLOBAL_TWEETS = [Peep; 10]

// The address for the registered user
AUTH = mapping Username -> Address
// All peeps by a single user
USER_TWEETS = mapping Username -> Vec<Peep>
// All users that this user is following
USER_FOLLOWS = mapping Username -> Vec<Username>
```

## API

**Note:** `caller()` returns the senders address.

```python
fn register(username)
    AUTH[username] = caller()

fn peep(username, peep)
    if AUTH[username] = caller()
        TWEETS[username].append(peep)

fn follow(username: String, followed: String)
    if AUTH[username] == caller() and AUTH[followed].is_some()
        FOLLOWING[username].append(followed)
```
