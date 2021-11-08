use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    fn message1(&self);

    #[ink(message, selector = 42)]
    fn message2(&self);

    #[ink(message, selector = 0xC0DECAFE)]
    fn message3(&mut self);
}

use ink_env::DefaultEnvironment;
use ink_lang::{
    reflect::{
        TraitDefinitionRegistry,
        TraitMessageInfo,
    },
    selector_bytes,
    selector_id,
};

fn main() {
    macro_rules! assert_selector_eq {
        ( $message_id:literal, $expected_selector:expr $(,)? ) => {
            assert_eq!(
                <<TraitDefinitionRegistry<DefaultEnvironment> as TraitDefinition>::__ink_TraitInfo
                    as TraitMessageInfo<{selector_id!($message_id)}>>::SELECTOR,
                    $expected_selector
            );
        }
    }

    assert_selector_eq!("message1", selector_bytes!("TraitDefinition::message1"),);
    assert_selector_eq!("message2", [0, 0, 0, 42],);
    assert_selector_eq!("message3", [0xC0, 0xDE, 0xCA, 0xFE],);
}
