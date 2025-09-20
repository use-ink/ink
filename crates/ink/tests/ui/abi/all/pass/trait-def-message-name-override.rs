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
        ( $message_id:expr, $expected_selector:expr $(,)? ) => {
            assert_eq!(
                <<TraitDefinitionRegistry<DefaultEnvironment> as TraitDefinition>::__ink_TraitInfo
                    as TraitMessageInfo<{$message_id}>>::SELECTOR,
                    $expected_selector
            );
        }
    }

    // ink! selector
    assert_selector_eq!(selector_id!("myMessage"), selector_bytes!("TraitDefinition::myMessage"));

    // `keccak256("myMessage()")` == `0x1b008a9f`
    assert_selector_eq!(0x1b008a9f_u32, [0x1b, 0x00, 0x8a, 0x9f]);
}
