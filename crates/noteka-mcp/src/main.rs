mod catalog;
mod server;
mod transport;

use noteka_core::scoped::{LocalScope, Policy};
use std::{path::PathBuf, time::Duration};
use tokio::time::Instant;

fn arguments(args: impl Iterator<Item = std::ffi::OsString>) -> Result<(PathBuf, Policy), ()> {
    let mut args = args;
    let mut path = None;
    let mut write = false;
    let mut run = false;
    while let Some(arg) = args.next() {
        if arg == "--workspace" && path.is_none() {
            path = Some(PathBuf::from(args.next().ok_or(())?));
        } else if arg == "--allow-write" && !write {
            write = true;
        } else if arg == "--allow-run" && !run {
            run = true;
        } else {
            return Err(());
        }
    }
    Ok((path.ok_or(())?, Policy::new(write, run).map_err(|_| ())?))
}
fn main() {
    let (path, policy) = match arguments(std::env::args_os().skip(1)) {
        Ok(v) => v,
        Err(()) => {
            eprintln!("Uso: noteka-mcp --workspace <ruta> [--allow-write [--allow-run]]");
            std::process::exit(2);
        }
    };
    let scope = match LocalScope::new(&path, policy) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("MCP: workspace_rejected ({:?}).", e.code);
            std::process::exit(2);
        }
    };
    let catalog = match catalog::Catalog::new(policy) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("MCP: catalog_error.");
            std::process::exit(2);
        }
    };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("local runtime");
    runtime.block_on(async move {
        let shared = transport::Shared::new(scope);
        let transport = transport::start(tokio::io::stdin(), tokio::io::stdout(), shared.clone());
        let mut signal = shared.signal();
        let service = server::Server {
            catalog,
            shared: shared.clone(),
        };
        let serve = async {
            match rmcp::service::serve_server(service, transport).await {
                Ok(running) => {
                    let _ = running.waiting().await;
                }
                Err(_) => {
                    if !shared.stopping() {
                        eprintln!("MCP: lifecycle_closed.");
                    }
                }
            }
        };
        tokio::select! { _=serve=>shared.shutdown(Instant::now()),_=signal.changed()=>() }
        let started = (*signal.borrow()).unwrap_or_else(Instant::now);
        shared.shutdown(started);
        // Do not use the runtime's unlimited spawn_blocking shutdown. The permit
        // stays inside synchronous work; supervisors observe scope.stop().
        let _ = tokio::time::timeout_at(
            started + Duration::from_secs(30),
            shared.core.clone().acquire_owned(),
        )
        .await;
    });
    runtime.shutdown_timeout(Duration::ZERO);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flags_are_explicit_and_immutable() {
        for args in [
            vec![],
            vec!["--workspace"],
            vec!["--workspace", "x", "--allow-run"],
            vec!["--workspace", "x", "--workspace", "y"],
            vec!["--workspace", "x", "--allow-write", "--allow-write"],
            vec!["--workspace", "x", "--port", "1"],
        ] {
            assert!(arguments(args.into_iter().map(Into::into)).is_err());
        }
        for tail in [
            vec![],
            vec!["--allow-write"],
            vec!["--allow-write", "--allow-run"],
        ] {
            assert!(
                arguments(["--workspace", "x"].into_iter().chain(tail).map(Into::into)).is_ok()
            );
        }
    }
}
