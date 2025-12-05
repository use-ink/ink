use crate::erc721::*;
use ink::primitives::Address;

fn set_caller(sender: Address) {
    ink::env::test::set_caller(sender);
}

#[ink::test]
fn mint_works() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Token 1 does not exists.
    assert_eq!(erc721.owner_of(1), None);
    // Alice does not owns tokens.
    assert_eq!(erc721.balance_of(accounts.alice), 0);
    // Create token Id 1.
    assert_eq!(erc721.mint(1), Ok(()));
    // Alice owns 1 token.
    assert_eq!(erc721.balance_of(accounts.alice), 1);
}

#[ink::test]
fn mint_existing_should_fail() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Create token Id 1.
    assert_eq!(erc721.mint(1), Ok(()));
    // The first Transfer event takes place
    assert_eq!(1, ink::env::test::recorded_events().len());
    // Alice owns 1 token.
    assert_eq!(erc721.balance_of(accounts.alice), 1);
    // Alice owns token Id 1.
    assert_eq!(erc721.owner_of(1), Some(accounts.alice));
    // Cannot create  token Id if it exists.
    // Bob cannot own token Id 1.
    assert_eq!(erc721.mint(1), Err(Error::TokenExists));
}

#[ink::test]
fn transfer_works() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Create token Id 1 for Alice
    assert_eq!(erc721.mint(1), Ok(()));
    // Alice owns token 1
    assert_eq!(erc721.balance_of(accounts.alice), 1);
    // Bob does not owns any token
    assert_eq!(erc721.balance_of(accounts.bob), 0);
    // The first Transfer event takes place
    assert_eq!(1, ink::env::test::recorded_events().len());
    // Alice transfers token 1 to Bob
    assert_eq!(erc721.transfer(accounts.bob, 1), Ok(()));
    // The second Transfer event takes place
    assert_eq!(2, ink::env::test::recorded_events().len());
    // Bob owns token 1
    assert_eq!(erc721.balance_of(accounts.bob), 1);
}

#[ink::test]
fn invalid_transfer_should_fail() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Transfer token fails if it does not exists.
    assert_eq!(erc721.transfer(accounts.bob, 2), Err(Error::TokenNotFound));
    // Token Id 2 does not exists.
    assert_eq!(erc721.owner_of(2), None);
    // Create token Id 2.
    assert_eq!(erc721.mint(2), Ok(()));
    // Alice owns 1 token.
    assert_eq!(erc721.balance_of(accounts.alice), 1);
    // Token Id 2 is owned by Alice.
    assert_eq!(erc721.owner_of(2), Some(accounts.alice));
    // Set Bob as caller
    set_caller(accounts.bob);
    // Bob cannot transfer not owned tokens.
    assert_eq!(erc721.transfer(accounts.eve, 2), Err(Error::NotApproved));
}

#[ink::test]
fn approved_transfer_works() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Create token Id 1.
    assert_eq!(erc721.mint(1), Ok(()));
    // Token Id 1 is owned by Alice.
    assert_eq!(erc721.owner_of(1), Some(accounts.alice));
    // Approve token Id 1 transfer for Bob on behalf of Alice.
    assert_eq!(erc721.approve(accounts.bob, 1), Ok(()));
    // Set Bob as caller
    set_caller(accounts.bob);
    // Bob transfers token Id 1 from Alice to Eve.
    assert_eq!(
        erc721.transfer_from(accounts.alice, accounts.eve, 1),
        Ok(())
    );
    // TokenId 3 is owned by Eve.
    assert_eq!(erc721.owner_of(1), Some(accounts.eve));
    // Alice does not owns tokens.
    assert_eq!(erc721.balance_of(accounts.alice), 0);
    // Bob does not owns tokens.
    assert_eq!(erc721.balance_of(accounts.bob), 0);
    // Eve owns 1 token.
    assert_eq!(erc721.balance_of(accounts.eve), 1);
}

#[ink::test]
fn approved_for_all_works() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Create token Id 1.
    assert_eq!(erc721.mint(1), Ok(()));
    // Create token Id 2.
    assert_eq!(erc721.mint(2), Ok(()));
    // Alice owns 2 tokens.
    assert_eq!(erc721.balance_of(accounts.alice), 2);
    // Approve token Id 1 transfer for Bob on behalf of Alice.
    assert_eq!(erc721.set_approval_for_all(accounts.bob, true), Ok(()));
    // Bob is an approved operator for Alice
    assert!(erc721.is_approved_for_all(accounts.alice, accounts.bob));
    // Set Bob as caller
    set_caller(accounts.bob);
    // Bob transfers token Id 1 from Alice to Eve.
    assert_eq!(
        erc721.transfer_from(accounts.alice, accounts.eve, 1),
        Ok(())
    );
    // TokenId 1 is owned by Eve.
    assert_eq!(erc721.owner_of(1), Some(accounts.eve));
    // Alice owns 1 token.
    assert_eq!(erc721.balance_of(accounts.alice), 1);
    // Bob transfers token Id 2 from Alice to Eve.
    assert_eq!(
        erc721.transfer_from(accounts.alice, accounts.eve, 2),
        Ok(())
    );
    // Bob does not own tokens.
    assert_eq!(erc721.balance_of(accounts.bob), 0);
    // Eve owns 2 tokens.
    assert_eq!(erc721.balance_of(accounts.eve), 2);
    // Remove operator approval for Bob on behalf of Alice.
    set_caller(accounts.alice);
    assert_eq!(erc721.set_approval_for_all(accounts.bob, false), Ok(()));
    // Bob is not an approved operator for Alice.
    assert!(!erc721.is_approved_for_all(accounts.alice, accounts.bob));
}

#[ink::test]
fn approve_nonexistent_token_should_fail() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Approve transfer of nonexistent token id 1
    assert_eq!(erc721.approve(accounts.bob, 1), Err(Error::TokenNotFound));
}

#[ink::test]
fn not_approved_transfer_should_fail() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Create token Id 1.
    assert_eq!(erc721.mint(1), Ok(()));
    // Alice owns 1 token.
    assert_eq!(erc721.balance_of(accounts.alice), 1);
    // Bob does not owns tokens.
    assert_eq!(erc721.balance_of(accounts.bob), 0);
    // Eve does not owns tokens.
    assert_eq!(erc721.balance_of(accounts.eve), 0);
    // Set Eve as caller
    set_caller(accounts.eve);
    // Eve is not an approved operator by Alice.
    assert_eq!(
        erc721.transfer_from(accounts.alice, accounts.frank, 1),
        Err(Error::NotApproved)
    );
    // Alice owns 1 token.
    assert_eq!(erc721.balance_of(accounts.alice), 1);
    // Bob does not owns tokens.
    assert_eq!(erc721.balance_of(accounts.bob), 0);
    // Eve does not owns tokens.
    assert_eq!(erc721.balance_of(accounts.eve), 0);
}

#[ink::test]
fn burn_works() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Create token Id 1 for Alice
    assert_eq!(erc721.mint(1), Ok(()));
    // Alice owns 1 token.
    assert_eq!(erc721.balance_of(accounts.alice), 1);
    // Alice owns token Id 1.
    assert_eq!(erc721.owner_of(1), Some(accounts.alice));
    // Destroy token Id 1.
    assert_eq!(erc721.burn(1), Ok(()));
    // Alice does not owns tokens.
    assert_eq!(erc721.balance_of(accounts.alice), 0);
    // Token Id 1 does not exists
    assert_eq!(erc721.owner_of(1), None);
}

#[ink::test]
fn burn_fails_token_not_found() {
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Try burning a non existent token
    assert_eq!(erc721.burn(1), Err(Error::TokenNotFound));
}

#[ink::test]
fn burn_fails_not_owner() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Create token Id 1 for Alice
    assert_eq!(erc721.mint(1), Ok(()));
    // Try burning this token with a different account
    set_caller(accounts.eve);
    assert_eq!(erc721.burn(1), Err(Error::NotOwner));
}

#[ink::test]
fn burn_clears_approval() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Create token Id 1 for Alice
    assert_eq!(erc721.mint(1), Ok(()));
    // Alice gives approval to Bob to transfer token Id 1
    assert_eq!(erc721.approve(accounts.bob, 1), Ok(()));
    // Alice burns token
    assert_eq!(erc721.burn(1), Ok(()));
    // Set caller to Frank
    set_caller(accounts.frank);
    // Frank mints token Id 1
    assert_eq!(erc721.mint(1), Ok(()));
    // Set caller to Bob
    set_caller(accounts.bob);
    // Bob tries to transfer token Id 1 from Frank to himself
    assert_eq!(
        erc721.transfer_from(accounts.frank, accounts.bob, 1),
        Err(Error::NotApproved)
    );
}

#[ink::test]
fn transfer_from_fails_not_owner() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Create token Id 1 for Alice
    assert_eq!(erc721.mint(1), Ok(()));
    // Bob can transfer alice's tokens
    assert_eq!(erc721.set_approval_for_all(accounts.bob, true), Ok(()));
    // Set caller to Frank
    set_caller(accounts.frank);
    // Create token Id 2 for Frank
    assert_eq!(erc721.mint(2), Ok(()));
    // Set caller to Bob
    set_caller(accounts.bob);
    // Bob makes invalid call to transfer_from (Alice is token owner, not Frank)
    assert_eq!(
        erc721.transfer_from(accounts.frank, accounts.bob, 1),
        Err(Error::NotOwner)
    );
}

#[ink::test]
fn transfer_fails_not_owner() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    // Create a new contract instance.
    let mut erc721 = Erc721::new();
    // Create token Id 1 for Alice
    assert_eq!(erc721.mint(1), Ok(()));
    // Bob can transfer alice's tokens
    assert_eq!(erc721.set_approval_for_all(accounts.bob, true), Ok(()));
    // Set caller to bob
    set_caller(accounts.bob);
    // Bob makes invalid call to transfer (he is not token owner, Alice is)
    assert_eq!(erc721.transfer(accounts.bob, 1), Err(Error::NotOwner));
}