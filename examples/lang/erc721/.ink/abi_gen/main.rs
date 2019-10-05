fn main() -> Result<(), std::io::Error> {
    let abi = contract::ink_generate_abi();
    let contents = serde_json::to_string_pretty(&abi)?;
    std::fs::create_dir("target").ok();
    std::fs::write("target/abi.json", contents)?;
    Ok(())
}
