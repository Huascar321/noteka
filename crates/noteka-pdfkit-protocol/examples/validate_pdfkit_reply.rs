use std::io::{Read, Write};
fn main() {
    let result = (|| {
        let args: Vec<_> = std::env::args().collect();
        if args.len() != 2 {
            return Err(());
        }
        let mut raw = Vec::new();
        std::io::stdin()
            .take(noteka_pdfkit_protocol::MAX_STDOUT as u64 + 1)
            .read_to_end(&mut raw)
            .map_err(|_| ())?;
        let reply = noteka_pdfkit_protocol::Reply::decode(&raw, &args[1]).map_err(|_| ())?;
        std::io::stdout()
            .write_all(&serde_json::to_vec(&reply).map_err(|_| ())?)
            .map_err(|_| ())
    })();
    if result.is_err() {
        eprintln!("pdfkit_worker_protocol");
        std::process::exit(1);
    }
}
