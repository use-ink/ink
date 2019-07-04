use ink_core::memory::vec::Vec;
use ink_utils;
use parity_codec::Encode;

pub trait EncodeSafe{
    fn encode_safe(&self) -> Vec<u8>;
}

macro_rules! impl_encode_safe {
	($( $one:ty ),*) => { $(
		impl EncodeSafe for $one {
			fn encode_safe(&self) -> Vec<u8> {
				parity_codec::Encode::encode(&self)
			}
		}
    )* }
}

impl_encode_safe!(bool,u16,u32,u64,u128,i8,i16,i32,i64,i128,Vec<u8>);

/// make selector from method name
#[allow(unused)]
fn raw_message_selector(name: &str) -> u32 {
    let keccak = ink_utils::hash::keccak256(name.as_bytes());
    u32::from_le_bytes([keccak[0], keccak[1], keccak[2], keccak[3]])
}

/// encode to input_data from Box wrapped params
#[allow(unused)]
pub fn gen_input_data(method: &str, params: &[Box<EncodeSafe>]) -> Vec<u8>
where
{
    let selector = raw_message_selector(method);
    let mut input_data = selector.encode();
    for param in params.iter() {
        let mut encoded_param = param.encode_safe();
        input_data.append(&mut encoded_param);
    }
    return input_data.encode()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_selectors() {
        assert_eq!(raw_message_selector("inc"), 257544423);
        assert_eq!(raw_message_selector("get"), 4266279973);
        assert_eq!(raw_message_selector("compare"), 363906316);
    }

    #[test]
    fn encode(){
        let var:bool = false;
        assert_eq!(var.encode(), vec![0]);

        let var:bool = true;
        assert_eq!(var.encode(), vec![1]);

        let var :u32 = 22;
        assert_eq!(var.encode(), vec![22,0,0,0]);

        let var :u32 = 257544423;
        assert_eq!(var.encode(), vec![231, 208, 89, 15]);

        let mut vec1 : Vec<u8> = Vec::new(); vec1.push(11);
        assert_eq!(vec1.encode(), vec![4,11]);
    }

    #[test]
    fn input_data_works(){
        let mut params : Vec<Box<EncodeSafe>> = Vec::new();
        params.push(Box::new(true));
        params.push(Box::new(12u64));
        assert_eq!(gen_input_data("inc",&params),vec![52, 231, 208, 89, 15, 1, 12, 0, 0, 0, 0, 0, 0, 0]);
    }
}
