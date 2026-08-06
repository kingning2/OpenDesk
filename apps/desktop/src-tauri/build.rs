use std::fs;
use std::path::PathBuf;

fn main() {
    ensure_external_bin_stub_for_dev("license-verifier");
    ensure_external_bin_stub_for_dev("opendesk-worker");
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

    // 1 字节占位即可，运行时通过大小跳过 stub（dev 流程会先构建真二进制覆盖它）。
    fs::write(&binary_path, [0]).expect("write externalBin stub");

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
