use super::Flip;
use super::conditional_compilation::ConditionalCompilation;

#[ink::test]
fn default_works() {
    let flipper = ConditionalCompilation::new();
    assert!(!flipper.get());
}

#[ink::test]
fn it_works() {
    let mut flipper = ConditionalCompilation::new();
    // Can call using universal call syntax using the trait.
    assert!(!<ConditionalCompilation as Flip>::get(&flipper));
    <ConditionalCompilation as Flip>::flip(&mut flipper);
    // Normal call syntax possible to as long as the trait is in scope.
    assert!(flipper.get());
}

#[cfg(feature = "foo")]
#[ink::test]
fn foo_works() {
    let mut flipper = ConditionalCompilation::new_foo(false);

    flipper.inherent_flip_foo();
    assert!(flipper.get());

    <ConditionalCompilation as Flip>::push_foo(&mut flipper, false);
    assert!(!flipper.get())
}