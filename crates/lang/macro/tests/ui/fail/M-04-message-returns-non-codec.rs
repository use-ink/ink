use ink_lang as ink;

#[ink::contract]
mod message_returns_non_codec {
    #[derive(scale_info::TypeInfo)]
    pub struct NonCodec;

    #[ink(storage)]
    pub struct MessageReturnsNonCodecType {}

    impl MessageReturnsNonCodecType {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn returns_non_codec_type(&self) -> NonCodec {
            NonCodec
        }
    }
}

fn main() {}
