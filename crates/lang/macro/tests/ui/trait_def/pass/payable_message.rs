use ink_lang as ink;

#[ink::trait_definition]
pub trait PayableDefinition {
    #[ink(message, payable)]
    fn payable(&self);
    #[ink(message, payable)]
    fn payable_mut(&mut self);

    #[ink(message)]
    fn unpayable(&self);
    #[ink(message)]
    fn unpayable_mut(&mut self);
}

/// Mimics computation of ink! trait local message ID calculation.
fn local_id(ident: &str) -> u32 {
    let buffer = format!("message::{}", ident).into_bytes();
    use blake2::digest::generic_array::sequence::Split as _;
    let (head_32, _rest) =
        <blake2::Blake2b as blake2::Digest>::digest(&buffer).split();
    let head_32: [u8; 4] = head_32.into();
    u32::from_be_bytes(head_32)
}

/// Computed using `local_id("payable")`.
const PAYABLE_ID: u32 = 0xFDDBE704;

/// Computed using `local_id("payable_mut")`.
const PAYABLE_MUT_ID: u32 = 0x1CE6275F;

/// Computed using `local_id("unpayable")`.
const UNPAYABLE_ID: u32 = 0x511647A5;

/// Computed using `local_id("unpayable_mut")`.
const UNPAYABLE_MUT_ID: u32 = 0x4A60F1E1;

fn main() {
    // Uncomment these in order to print out the local IDs of
    // all the ink! trait messages for this test.
    //
    // println!("local_id(\"payable\")       = {:X}", local_id("payable"));
    // println!("local_id(\"payable_mut\")   = {:X}", local_id("payable_mut"));
    // println!("local_id(\"unpayable\")     = {:X}", local_id("unpayable"));
    // println!("local_id(\"unpayable_mut\") = {:X}", local_id("unpayable_mut"));
    assert!(
        <<::ink_lang::TraitCallForwarderRegistry<::ink_env::DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as ::ink_lang::TraitMessageInfo<PAYABLE_ID>>::PAYABLE,
    );
    assert!(
        <<::ink_lang::TraitCallForwarderRegistry<::ink_env::DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as ::ink_lang::TraitMessageInfo<PAYABLE_MUT_ID>>::PAYABLE,
    );
    assert_eq!(
        <<::ink_lang::TraitCallForwarderRegistry<::ink_env::DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as ::ink_lang::TraitMessageInfo<UNPAYABLE_ID>>::PAYABLE,
        false
    );
    assert_eq!(
        <<::ink_lang::TraitCallForwarderRegistry<::ink_env::DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as ::ink_lang::TraitMessageInfo<UNPAYABLE_MUT_ID>>::PAYABLE,
        false
    );
}
