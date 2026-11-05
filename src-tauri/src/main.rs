#[cfg(all(target_os = "macos", debug_assertions, not(feature = "native-smoke")))]
mod dev_bundle;

fn main() {
    #[cfg(all(target_os = "macos", debug_assertions, not(feature = "native-smoke")))]
    if let Err(error) = dev_bundle::relaunch_if_unbundled() {
        eprintln!("Could not prepare Noteka's macOS development app: {error}");
        std::process::exit(1);
    }
    #[cfg(all(target_os = "macos", debug_assertions, not(feature = "native-smoke")))]
    dev_bundle::watch_supervisor();
    noteka_desktop::run();
    #[cfg(all(target_os = "macos", debug_assertions, not(feature = "native-smoke")))]
    dev_bundle::cleanup();
}
