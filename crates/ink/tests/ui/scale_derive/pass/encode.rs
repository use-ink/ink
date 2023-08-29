#[ink::scale_derive(encode)]
struct S;

fn is_encode<T: ::ink::scale::Encode>(_: T) {}

fn main() {
    is_encode(S);
}