//! Único módulo unsafe del helper: allocator, límites POSIX y salida sin asignación.
//! Se contabilizan tamaños Layout solicitados vivos, no RSS ni overhead del malloc.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
    time::{Duration, Instant},
};

pub const HEAP_BYTES: usize = 128 * 1024 * 1024;
pub const EXIT_MEMORY: i32 = 73;
pub const EXIT_DEADLINE: i32 = 75;
pub const EXIT_PANIC: i32 = 74;
pub const EXIT_SETUP: i32 = 70;
static REMAINING_MS: AtomicU32 = AtomicU32::new(5_000);

pub struct BoundedAllocator {
    live: AtomicUsize,
    peak: AtomicUsize,
}

impl BoundedAllocator {
    pub const fn new() -> Self {
        Self {
            live: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        }
    }
    pub fn live_bytes(&self) -> usize {
        self.live.load(Ordering::Acquire)
    }
    pub fn peak_bytes(&self) -> usize {
        self.peak.load(Ordering::Acquire)
    }

    fn reserve(&self, bytes: usize) -> bool {
        if let Ok(previous) = self
            .live
            .try_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                value.checked_add(bytes).filter(|&next| next <= HEAP_BYTES)
            })
        {
            self.peak.fetch_max(previous + bytes, Ordering::AcqRel);
            true
        } else {
            false
        }
    }
    fn release(&self, bytes: usize) {
        self.live.fetch_sub(bytes, Ordering::AcqRel);
    }
}
impl Default for BoundedAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: las reservas usan CAS y sumas comprobadas antes de llamar a System.
// Se delegan exactamente los mismos Layout/punteros válidos recibidos por GlobalAlloc.
// Realloc creciente reserva sólo el delta; al fallar System se revierte el delta.
// Realloc decreciente libera sólo después del éxito. No hay unwind, IO o formatting
// en estos caminos; el fallo termina el proceso sin llamar a destructores/allocators.
unsafe impl GlobalAlloc for BoundedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !self.reserve(layout.size()) {
            terminate(EXIT_MEMORY);
        }
        // SAFETY: el contrato GlobalAlloc garantiza un Layout válido/no cero.
        let pointer = unsafe { System.alloc(layout) };
        if pointer.is_null() {
            self.release(layout.size());
            terminate(EXIT_MEMORY);
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if !self.reserve(layout.size()) {
            terminate(EXIT_MEMORY);
        }
        // SAFETY: igual a alloc; System preserva alineación e inicializa todos los bytes.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if pointer.is_null() {
            self.release(layout.size());
            terminate(EXIT_MEMORY);
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: el llamador garantiza pointer/layout de una asignación viva de este allocator.
        unsafe {
            System.dealloc(pointer, layout);
        }
        self.release(layout.size());
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let growth = new_size.saturating_sub(layout.size());
        if !self.reserve(growth) {
            terminate(EXIT_MEMORY);
        }
        // SAFETY: el contrato GlobalAlloc garantiza pointer/Layout vivos y new_size > 0.
        let next = unsafe { System.realloc(pointer, layout, new_size) };
        if next.is_null() {
            self.release(growth);
            terminate(EXIT_MEMORY);
        }
        self.release(layout.size().saturating_sub(new_size));
        next
    }
}

#[global_allocator]
pub static ALLOCATOR: BoundedAllocator = BoundedAllocator::new();

pub fn terminate(code: i32) -> ! {
    // SAFETY: _exit termina todo el proceso sin buffers/destructores ni asignaciones.
    unsafe {
        libc::_exit(code);
    }
}

/// Se ejecuta antes de leer bytes del original; cualquier fallo impide procesamiento.
pub fn initialize() -> Instant {
    let started = Instant::now();
    // SAFETY: setrlimit recibe punteros válidos a rlimit; modifica sólo este proceso.
    let ok = unsafe {
        libc::setrlimit(
            libc::RLIMIT_CORE,
            &libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            },
        ) == 0
            && libc::setrlimit(
                libc::RLIMIT_CPU,
                &libc::rlimit {
                    rlim_cur: 4,
                    rlim_max: 4,
                },
            ) == 0
    };
    if !ok {
        terminate(EXIT_SETUP);
    }
    std::panic::set_hook(Box::new(|_| terminate(EXIT_PANIC)));
    if std::thread::Builder::new()
        .name("parser-budget".into())
        .stack_size(64 * 1024)
        .spawn(move || {
            loop {
                let duration =
                    Duration::from_millis(u64::from(REMAINING_MS.load(Ordering::Acquire)));
                if started.elapsed() >= duration {
                    terminate(EXIT_DEADLINE);
                }
                std::thread::sleep(
                    Duration::from_millis(5).min(duration.saturating_sub(started.elapsed())),
                );
            }
        })
        .is_err()
    {
        terminate(EXIT_SETUP);
    }
    started
}

/// Nunca extiende el watchdog inicial. El presupuesto empieza al iniciar el helper.
pub fn shorten_budget(remaining_ms: u32) {
    REMAINING_MS.fetch_min(remaining_ms, Ordering::AcqRel);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reservations_accept_frontier_reject_overflow_and_roll_back() {
        let allocator = BoundedAllocator::new();
        assert!(allocator.reserve(HEAP_BYTES));
        assert!(!allocator.reserve(1));
        assert!(!allocator.reserve(usize::MAX));
        assert_eq!(allocator.live_bytes(), HEAP_BYTES);
        allocator.release(HEAP_BYTES);
        assert!(allocator.reserve(16));
        allocator.release(16);
        assert_eq!(allocator.live_bytes(), 0);
        assert_eq!(allocator.peak_bytes(), HEAP_BYTES);
    }
    #[test]
    fn concurrent_reservations_never_cross_the_cap() {
        let allocator = BoundedAllocator::new();
        std::thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| {
                    for _ in 0..10_000 {
                        if allocator.reserve(HEAP_BYTES / 4) {
                            assert!(allocator.live_bytes() <= HEAP_BYTES);
                            std::thread::yield_now();
                            allocator.release(HEAP_BYTES / 4);
                        }
                    }
                });
            }
        });
        assert_eq!(allocator.live_bytes(), 0);
        assert!(allocator.peak_bytes() <= HEAP_BYTES);
    }
}
