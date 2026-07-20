use std::fs;
use std::path::PathBuf;

fn main() {
    ensure_external_bin_stub_for_dev("sidecar");
    ensure_external_bin_stub_for_dev("license-verifier");
    tauri_build::build();
}

/// Tauri validates `externalBin` at compile time. Release builds must run
/// the matching build script first; debug/clippy only needs a placeholder file.
fn ensure_external_bin_stub_for_dev(base_name: &str) {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    if profile == "release" {
        return;
    }

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binaries_dir = manifest_dir.join("binaries");
    let binary_path = binaries_dir.join(external_binary_name(base_name, &target));

    if binary_path.is_file() {
        println!("cargo:rerun-if-changed={}", binary_path.display());
        return;
    }

    fs::create_dir_all(&binaries_dir).expect("create binaries directory");

    #[cfg(unix)]
    write_unix_stub(&binary_path);

    #[cfg(windows)]
    write_windows_stub(&binary_path);

    println!(
        "cargo:warning=created dev {base_name} stub at {}",
        binary_path.display()
    );
    println!("cargo:rerun-if-changed={}", binaries_dir.display());
}

fn external_binary_name(base_name: &str, target: &str) -> String {
    let base = format!("{base_name}-{target}");
    if target.contains("windows") {
        format!("{base}.exe")
    } else {
        base
    }
}

#[cfg(unix)]
fn write_unix_stub(path: &PathBuf) {
    use std::os::unix::fs::PermissionsExt;

    const STUB: &[u8] = b"#!/bin/sh\nexit 0\n";
    fs::write(path, STUB).expect("write externalBin stub");
    let mut permissions = fs::metadata(path)
        .expect("externalBin stub metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("chmod externalBin stub");
}

#[cfg(windows)]
fn write_windows_stub(path: &PathBuf) {
    if let Ok(system_root) = std::env::var("SystemRoot") {
        let donor = PathBuf::from(system_root)
            .join("System32")
            .join("where.exe");
        if donor.is_file() {
            fs::copy(donor, path).expect("copy externalBin stub");
            return;
        }
    }

    fs::write(path, [0]).expect("write externalBin stub");
}
