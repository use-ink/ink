use contract::Contract;

#[ink::contract(abi = "sol")]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message_0(&self, _input_1: bool) {}

        #[ink(message, selector = 1)]
        pub fn message_1(&self, _input_1: i8) {}

        #[ink(message, selector = 0xC0DE_CAFE)]
        pub fn message_2(&self, _input_1: [u8; 32]) {}

        #[ink(message)]
        pub fn message_3(&self, _input_1: bool, _input_2: i8) {}
    }

    #[ink::trait_definition]
    pub trait Messages {
        #[ink(message, selector = 0x12345678)]
        fn message_4(&self, _input_1: Vec<u8>);
    }

    impl Messages for Contract {
        #[ink(message, selector = 0x12345678)]
        fn message_4(&self, _input_1: Vec<u8>) {}
    }

    impl Contract {
        #[ink(message)]
        pub fn message_5(&self, _input_1: String) {}

        #[ink(message)]
        pub fn message_6(&self, _input_1: bool, _input_2: String, _input_3: [u8; 32], _input_4: Vec<u8>, _input_5: [u16; 4], _input_6: Vec<u16>) {}
    }
}

fn main() {
    // `keccak256("message_0(bool)")` == `0xc58b7175`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xc58b7175_u32>>::SELECTOR,
        [0xc5, 0x8b, 0x71, 0x75],
    );

    // `keccak256("message_1(int8)")` == `0xb316130f`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xb316130f_u32>>::SELECTOR,
        [0xb3, 0x16, 0x13, 0x0f],
    );

    // `keccak256("message_2(bytes32)")` == `0x468f916c`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x468f916c_u32>>::SELECTOR,
        [0x46, 0x8f, 0x91, 0x6c],
    );

    // `keccak256("message_3(bool,int8)")` == `0xf4bf21e5`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xf4bf21e5_u32>>::SELECTOR,
        [0xf4, 0xbf, 0x21, 0xe5],
    );

    // `keccak256("message_4(bytes)")` == `0x0e59eb1b`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x0e59eb1b_u32>>::SELECTOR,
        [0x0e, 0x59, 0xeb, 0x1b],
    );

    // `keccak256("message_5(string)")` == `0x596379bc`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x596379bc_u32>>::SELECTOR,
        [0x59, 0x63, 0x79, 0xbc],
    );

    // `keccak256("message_6(bool,string,bytes32,bytes,uint16[4],uint16[])")` == `0x270af153`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x270af153_u32>>::SELECTOR,
        [0x27, 0x0a, 0xf1, 0x53],
    );
}
