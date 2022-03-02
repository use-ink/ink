use ink_lang as ink;

#[ink::trait_definition(keep_attr = "foo, bar")]
pub trait WithKeepAttr {
    #[ink(message)]
    #[allow(non_snake_case)]
    fn meSSage(&self);
}

fn main() {}
