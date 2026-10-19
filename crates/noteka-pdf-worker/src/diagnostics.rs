//! Los argumentos del parser nunca se formatean, imprimen ni conservan.
use std::{cell::Cell, sync::OnceLock};

thread_local! {
    static DEGRADED: Cell<bool> = const { Cell::new(false) };
}
struct ClosedLogger;
static LOGGER: ClosedLogger = ClosedLogger;
static INSTALLED: OnceLock<bool> = OnceLock::new();

impl log::Log for ClosedLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Warn
    }
    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            DEGRADED.set(true);
        }
    }
    fn flush(&self) {}
}

pub fn begin() -> bool {
    let installed = *INSTALLED.get_or_init(|| {
        if log::set_logger(&LOGGER).is_err() {
            return false;
        }
        log::set_max_level(log::LevelFilter::Warn);
        true
    });
    DEGRADED.set(false);
    installed
}

pub fn degraded() -> bool {
    DEGRADED.get()
}

#[cfg(test)]
mod tests {
    #[test]
    fn diagnostics_do_not_format_arguments() {
        assert!(super::begin());
        struct Secret;
        impl std::fmt::Display for Secret {
            fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                panic!("el logger no debe evaluar argumentos");
            }
        }
        log::warn!("{}", Secret);
        assert!(super::degraded());
        assert!(super::begin());
        assert!(!super::degraded());
    }
}
