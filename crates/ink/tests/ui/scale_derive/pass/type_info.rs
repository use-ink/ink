#[ink::scale_derive(type_info)]
struct S;

fn is_type_info<T: ::ink::scale_info::TypeInfo>(_: T) {}

fn main() {
    is_type_info(S);
}