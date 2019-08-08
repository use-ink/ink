
fn main() -> Result<(), std::io::Error> {
    let abi = contract::ink_generate_abi();
    let contents = serde_json::to_string_pretty(&abi)?;
        // .map_err(|_| "failed at converting contract ABI to JSON")?;
    std::fs::create_dir("target").ok();
        // .map_err(|_| "failed to create `target` directory")?;
    std::fs::write("target/abi.json", contents)?;
        // .map_err(|_| "failed at writing JSON ABI to file")?;
    Ok(())
}
