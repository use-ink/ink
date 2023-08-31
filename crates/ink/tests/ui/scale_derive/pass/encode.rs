#[ink::scale_derive(Encode)]
struct S;

fn is_encode<T: ::ink::scale::Encode>(_: T) {}

fn main() {
    is_encode(S);
}