extern crate contract;

extern "C" {
    fn generate_metadata() -> i32;
}

fn main() -> Result<(), std::io::Error> {
    unsafe { generate_metadata(); }
    Ok(())
}
