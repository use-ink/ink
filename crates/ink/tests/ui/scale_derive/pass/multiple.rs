#[ink::scale_derive(encode, decode, type_info)]
struct S;

fn is_type_info<T: ::ink::scale_info::TypeInfo>(_: T) {}
fn is_encode<T: ::ink::scale::Encode>(_: T) {}
fn is_decode<T: ::ink::scale::Decode>(_: T) {}

fn main() {
    is_type_info(S);
    is_encode(S);
    is_decode(S);
}