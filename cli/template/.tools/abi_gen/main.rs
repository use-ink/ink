

fn main() {
    let abi = contract::generate_abi();
    let contents = serde_json::to_string(&abi)
        .expect("Failed at converting contract ABI to JSON");
    let mut path_buf = String::from("target/");
    path_buf.push_str(description.name());
    path_buf.push_str(".json");
    std::fs::create_dir("target")
		.expect("Failed to create target directory");
    std::fs::write(path_buf, contents)
        .expect("Failed at writing JSON ABI to file");
    Ok(())
}
