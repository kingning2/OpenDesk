//! Adapter build：嵌入 verifier SHA-256、attestation 密钥，以及 Windows MSVC 目标 triple。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target = license_target_triple();
    println!("cargo:rustc-env=OPENDESK_LICENSE_TARGET_TRIPLE={target}");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let generated = manifest_dir.join("generated");
    println!(
        "cargo:rerun-if-changed={}",
        generated.join("license_verifier.sha256").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        generated.join("license_attest_key.hex").display()
    );

    let sha = read_trimmed(generated.join("license_verifier.sha256"));
    let attest = read_trimmed(generated.join("license_attest_key.hex"));
    println!("cargo:rustc-env=OPENDESK_LICENSE_VERIFIER_SHA256={sha}");
    println!("cargo:rustc-env=OPENDESK_LICENSE_ATTEST_KEY_HEX={attest}");
}

/// Windows 上固定映射为 `*-windows-msvc`，与 bundled verifier 命名一致。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
fn license_target_triple() -> String {
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    if cfg!(target_os = "windows") || target.contains("windows") {
        if target.contains("windows-gnu") {
            return target.replace("windows-gnu", "windows-msvc");
        }
        if target.contains("windows-msvc") {
            return target;
        }
        return "x86_64-pc-windows-msvc".into();
    }
    target
}

fn read_trimmed(path: PathBuf) -> String {
    fs::read_to_string(&path)
        .map(|value| value.trim().to_string())
        .unwrap_or_default()
}
