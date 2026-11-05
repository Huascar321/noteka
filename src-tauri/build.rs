fn main() {
    println!("cargo:rerun-if-changed=native/location.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        let object = out.join("noteka-location.o");
        let archive = out.join("libnoteka_location.a");
        let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        let arch = if arch == "aarch64" { "arm64" } else { "x86_64" };
        let status = std::process::Command::new("xcrun").args(["clang", "-fobjc-arc", "-fblocks", "-arch", arch, "-c", "native/location.m", "-o"]).arg(&object).status().expect("clang unavailable");
        assert!(status.success(), "location compilation failed");
        let status = std::process::Command::new("xcrun").args(["ar", "crus"]).arg(&archive).arg(&object).status().expect("ar unavailable");
        assert!(status.success(), "location archive failed");
        println!("cargo:rustc-link-search=native={}", out.display());
        println!("cargo:rustc-link-lib=static=noteka_location");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=CoreLocation");
    }
    tauri_build::build()
}
