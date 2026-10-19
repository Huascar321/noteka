//! Probe sólo de desarrollo: no es protocolo/argumento del helper de producción.
use noteka_pdf_worker::containment::{self, ALLOCATOR, BoundedAllocator, HEAP_BYTES};
use std::{
    alloc::{GlobalAlloc, Layout},
    io::{Read, Write},
    time::Duration,
};

fn main() {
    containment::initialize();
    match std::env::args().nth(1).as_deref() {
        Some("accounting") => accounting(),
        Some("concurrent") => concurrent(),
        Some("limits") => {
            let mut cpu = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            let mut core = cpu;
            // SAFETY: buffers rlimit válidos; getrlimit sólo lee límites del proceso.
            unsafe {
                assert_eq!(libc::getrlimit(libc::RLIMIT_CPU, &mut cpu), 0);
                assert_eq!(libc::getrlimit(libc::RLIMIT_CORE, &mut core), 0);
            }
            println!(
                "{} {} {} {}",
                cpu.rlim_cur, cpu.rlim_max, core.rlim_cur, core.rlim_max
            );
        }
        Some("oom") => {
            let layout = Layout::from_size_align(HEAP_BYTES + 1, 8).unwrap();
            // SAFETY: Layout válido, tamaño no cero; se espera salida antes de System.
            unsafe {
                ALLOCATOR.alloc(layout);
            }
            unreachable!();
        }
        Some("realloc-oom") => {
            let layout = Layout::from_size_align(16, 8).unwrap();
            // SAFETY: pointer/layout de una asignación viva del mismo allocator.
            unsafe {
                let pointer = ALLOCATOR.alloc(layout);
                ALLOCATOR.realloc(pointer, layout, HEAP_BYTES + 1);
            }
            unreachable!();
        }
        Some("panic") => panic!("THIS_PRIVATE_CONTENT_MUST_NOT_ESCAPE"),
        Some("abort") => std::process::abort(),
        Some("cpu") => {
            std::io::stdout().write_all(b"ready\n").unwrap();
            std::io::stdout().flush().unwrap();
            loop {
                std::hint::black_box(17u64.wrapping_mul(113));
            }
        }
        Some("parent") => {
            let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                .arg("hang")
                .stdin(std::process::Stdio::null())
                // El harness ve EOF sólo cuando también termina el child tras SIGKILL del padre.
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap();
            println!("pid={}", child.id());
            std::io::stdout().flush().unwrap();
            // El harness mata este padre; no debe dejar el child sin watchdog propio.
            child.wait().unwrap();
        }
        Some("hang") => {
            println!("ready");
            std::io::stdout().flush().unwrap();
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        Some("short-budget") => {
            containment::shorten_budget(100);
            containment::shorten_budget(5_000);
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        Some("input") => {
            let mut byte = [0];
            std::io::stdin().read_exact(&mut byte).unwrap();
        }
        _ => containment::terminate(containment::EXIT_SETUP),
    }
}

fn accounting() {
    let allocator = BoundedAllocator::new();
    let first = Layout::from_size_align(64, 64).unwrap();
    // SAFETY: cada Layout se corresponde con la asignación/realloc viva. Se leen
    // únicamente bytes inicializados; se usa el mismo allocator y la alineación 64.
    unsafe {
        let pointer = allocator.alloc_zeroed(first);
        assert_eq!((pointer as usize) % 64, 0);
        assert_eq!(std::slice::from_raw_parts(pointer, 64), &[0; 64]);
        assert_eq!(allocator.live_bytes(), 64);
        let pointer = allocator.realloc(pointer, first, 128);
        assert_eq!(allocator.live_bytes(), 128);
        let pointer = allocator.realloc(pointer, Layout::from_size_align(128, 64).unwrap(), 32);
        assert_eq!(allocator.live_bytes(), 32);
        allocator.dealloc(pointer, Layout::from_size_align(32, 64).unwrap());
        assert_eq!(allocator.live_bytes(), 0);
        let pointer = allocator.alloc(first);
        std::ptr::write_bytes(pointer, 0xab, 64);
        allocator.dealloc(pointer, first);
        assert_eq!(allocator.live_bytes(), 0);
        assert_eq!(allocator.peak_bytes(), 128);
    }
    println!("{} {}", ALLOCATOR.live_bytes(), ALLOCATOR.peak_bytes());
}

fn concurrent() {
    let allocator = BoundedAllocator::new();
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let allocator = &allocator;
            scope.spawn(move || {
                for _ in 0..1000 {
                    let first = Layout::from_size_align(8192, 64).unwrap();
                    // SAFETY: cada hilo mantiene su propio pointer vivo; sólo comparte
                    // la cuenta atómica. Layout/alineación corresponden a cada realloc.
                    unsafe {
                        let pointer = allocator.alloc_zeroed(first);
                        assert_eq!(std::slice::from_raw_parts(pointer, 64), &[0; 64]);
                        let pointer = allocator.realloc(pointer, first, 16384);
                        assert_eq!(std::slice::from_raw_parts(pointer, 64), &[0; 64]);
                        let pointer = allocator.realloc(
                            pointer,
                            Layout::from_size_align(16384, 64).unwrap(),
                            4096,
                        );
                        allocator.dealloc(pointer, Layout::from_size_align(4096, 64).unwrap());
                    }
                }
            });
        }
    });
    assert_eq!(allocator.live_bytes(), 0);
    assert!(allocator.peak_bytes() <= HEAP_BYTES);
    println!("{}", allocator.peak_bytes());
}
