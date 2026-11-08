//! macOS development needs an application bundle so Location Services can identify it.
use std::{
    fs, io,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn relaunch_if_unbundled() -> io::Result<()> {
    let executable = std::env::current_exe()?;
    let executable_dir = executable
        .parent()
        .ok_or_else(|| io::Error::other("Executable directory missing"))?;
    if executable_dir.ends_with("Contents/MacOS") {
        return Ok(());
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let root: PathBuf =
        std::env::temp_dir().join(format!("noteka-dev-{}-{stamp}", std::process::id()));
    let app = root.join("Noteka.app");
    let contents = app.join("Contents");
    let macos = contents.join("MacOS");
    fs::create_dir_all(&macos)?;
    let target = macos.join("noteka-desktop");
    fs::copy(&executable, &target)?;
    for name in [
        "noteka-pdf-worker",
        "noteka-xlsx-worker",
        "noteka-ocr-worker",
        "noteka-pdfkit-worker",
        "noteka-pdf-raster-worker",
        "noteka-pdfium-worker",
    ] {
        let source = executable_dir.join(name);
        if source.is_file() {
            fs::copy(source, macos.join(name))?;
        }
    }
    let info = include_str!("../Info.plist").replace("</dict>", "<key>CFBundleIdentifier</key><string>app.noteka.desktop</string><key>CFBundleExecutable</key><string>noteka-desktop</string><key>CFBundleName</key><string>Noteka</string><key>CFBundlePackageType</key><string>APPL</string></dict>");
    fs::write(contents.join("Info.plist"), info)?;
    write_icon_resources(&contents)?;
    let status = Command::new("/usr/bin/codesign")
        .args(["--force", "--sign", "-"])
        .arg(&app)
        .status()?;
    if !status.success() {
        return Err(io::Error::other("Could not sign the local development app"));
    }
    // Launch Services must start the bundle; executing its binary directly does
    // not reliably establish the application identity used by Core Location.
    // `open -W` replaces this process, preserving the PID watched by Cargo.
    let supervisor = format!("NOTEKA_DEV_SUPERVISOR={}", std::process::id());
    let error = Command::new("/usr/bin/open")
        .args(["-n", "-W", "--env"])
        .arg(supervisor)
        .arg(&app)
        .arg("--args")
        .args(std::env::args_os().skip(1))
        .exec();
    Err(error)
}

// Development bundles need the same native icon resources as packaged builds.
// Embed them so a relocated binary does not depend on the source checkout.
fn write_icon_resources(contents: &Path) -> io::Result<()> {
    let resources = contents.join("Resources");
    fs::create_dir_all(&resources)?;
    fs::write(
        resources.join("Assets.car"),
        include_bytes!("../icons/Assets.car"),
    )?;
    fs::write(
        resources.join("icon.icns"),
        include_bytes!("../icons/icon.icns"),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_bundle_contains_native_and_fallback_icon_resources() {
        let root = std::env::temp_dir().join(format!("noteka-icon-test-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let result = (|| -> io::Result<()> {
            let contents = root.join("Noteka.app/Contents");
            write_icon_resources(&contents)?;
            assert_eq!(
                fs::read(contents.join("Resources/Assets.car"))?,
                include_bytes!("../icons/Assets.car")
            );
            assert_eq!(
                fs::read(contents.join("Resources/icon.icns"))?,
                include_bytes!("../icons/icon.icns")
            );
            let info = include_str!("../Info.plist");
            assert!(info.contains("<key>CFBundleIconName</key><string>Noteka</string>"));
            assert!(info.contains("<key>CFBundleIconFile</key><string>icon.icns</string>"));
            Ok(())
        })();
        fs::remove_dir_all(&root).unwrap();
        result.unwrap();
    }
}

pub fn watch_supervisor() {
    let Some(pid) = std::env::var("NOTEKA_DEV_SUPERVISOR")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .filter(|pid| *pid > 1)
    else {
        return;
    };
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            unsafe extern "C" {
                fn kill(pid: i32, signal: i32) -> i32;
            }
            if unsafe { kill(pid, 0) } != 0 {
                cleanup();
                std::process::exit(0);
            }
        }
    });
}

pub fn cleanup() {
    if std::env::var("NOTEKA_DEV_SUPERVISOR")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .is_none()
    {
        return;
    }
    let Some(root) = std::env::current_exe().ok().and_then(|exe| {
        exe.parent()?
            .parent()?
            .parent()?
            .parent()
            .map(|p| p.to_path_buf())
    }) else {
        return;
    };
    let Ok(temp) = std::env::temp_dir().canonicalize() else {
        return;
    };
    let Ok(root) = root.canonicalize() else {
        return;
    };
    if root.parent() == Some(temp.as_path())
        && root
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("noteka-dev-"))
    {
        let _ = fs::remove_dir_all(root);
    }
}
