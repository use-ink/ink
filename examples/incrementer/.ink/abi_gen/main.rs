extern crate contract;

extern "Rust" {
    fn generate_metadata() -> ink_abi::InkProject;
}

fn main() -> Result<(), std::io::Error> {
    let ink_project = unsafe { generate_metadata() };
    let contents = serde_json::to_string_pretty(&ink_project)?;
    std::fs::create_dir("target").ok();
    std::fs::write("target/metadata.json", contents)?;
    Ok(())
}
