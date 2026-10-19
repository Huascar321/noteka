use std::{
    io::{Read, Write},
    path::Path,
};
fn main() {
    let result =
        (|| {
            let args: Vec<_> = std::env::args().collect();
            if args.len() != 2 {
                return Err(());
            }
            let mut source = Vec::new();
            std::io::stdin()
                .take(noteka_pdfkit_protocol::MAX_SOURCE as u64 + 1)
                .read_to_end(&mut source)
                .map_err(|_| ())?;
            let output =
                match noteka_pdfkit_runtime::read(Path::new(&args[1]), &source, 5000, || {
                    Ok::<_, ()>(())
                }) {
                    Ok(document) => {
                        let prepared =
                            noteka_pdfkit_runtime::canonical::prepare(document).map_err(|_| ())?;
                        let loaded = noteka_pdfkit_runtime::canonical::load(
                            &prepared.chunks,
                            &prepared.representation_sha256,
                            prepared.representation_bytes,
                        )
                        .map_err(|_| ())?;
                        serde_json::to_value(loaded.document).map_err(|_| ())?
                    }
                    Err(noteka_pdfkit_runtime::Error::Rejected(reason)) => {
                        serde_json::json!({"status":"error","reason":reason})
                    }
                    Err(_) => return Err(()),
                };
            std::io::stdout()
                .write_all(&serde_json::to_vec(&output).map_err(|_| ())?)
                .map_err(|_| ())
        })();
    if result.is_err() {
        eprintln!("pdfkit_runtime_failed");
        std::process::exit(1);
    }
}
