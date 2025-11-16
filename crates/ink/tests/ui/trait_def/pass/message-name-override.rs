use ink::{
    reflect::{
        TraitDefinitionRegistry,
        TraitMessageInfo,
    },
    selector_bytes,
    selector_id,
};
use ink_env::DefaultEnvironment;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message, name = "myMessage")]
    fn message(&self);
}

fn main() {
    // Message selector and selector id both use the name override.
    macro_rules! assert_selector_eq {
        ( $message_id:literal, $expected_selector:expr $(,)? ) => {
            assert_eq!(
                <<TraitDefinitionRegistry<DefaultEnvironment> as TraitDefinition>::__ink_TraitInfo
                    as TraitMessageInfo<{selector_id!(Abi::Ink, $message_id)}>>::SELECTOR,
                    $expected_selector
            );
        }
    }

    assert_selector_eq!(
        "myMessage",
        selector_bytes!(Abi::Ink, "TraitDefinition::myMessage"),
    );
}
