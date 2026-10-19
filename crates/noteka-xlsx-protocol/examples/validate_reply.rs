//! Probe de desarrollo del validador que usará el futuro supervisor.
use std::io::Read;
fn main() {
    let hash = std::env::args().nth(1).unwrap_or_default();
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(noteka_xlsx_protocol::MAX_STDOUT as u64 + 1)
        .read_to_end(&mut bytes)
        .unwrap();
    if noteka_xlsx_protocol::Reply::decode(&bytes, &hash).is_err() {
        std::process::exit(2);
    }
}
