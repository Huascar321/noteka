fn main() {
    println!(
        "{}",
        serde_json::to_string_pretty(&noteka_core::contract_schema()).unwrap()
    );
}
