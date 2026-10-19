//! Sólo medición: mismo entrypoint/IPC/IO/allocator que el helper operativo.
//! El resultado normal sigue en stdout; las métricas separadas van a stderr.
use noteka_pdf_worker::{containment, runtime};
fn main() {
    let began = std::time::Instant::now();
    runtime::run();
    let heap = containment::ALLOCATOR.peak_bytes();
    // SAFETY: rusage propio inicializado y escribible, sin modificar límites.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    assert_eq!(unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) }, 0);
    let rss = usage.ru_maxrss as u64 * if cfg!(target_os = "macos") { 1 } else { 1024 };
    eprintln!(
        "{}",
        serde_json::json!({"elapsed_ms":began.elapsed().as_secs_f64()*1000.0,"heap_peak_requested_bytes":heap,"peak_rss_bytes":rss})
    );
}
