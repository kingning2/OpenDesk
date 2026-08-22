//! infra build：嵌入 license verifier SHA / attestation 密钥 / 目标 triple。
//!
//! 合并自原 `crates/adapter`（license 校验数据）与 `crates/runtime`（sidecar 目标 triple）的 build.rs。

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // 原 crates/runtime：sidecar 命名用目标 triple。
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=DINGDA_TARGET_TRIPLE={target}");

    // 原 crates/adapter：license verifier 校验数据。
    let license_triple = license_target_triple(&target);
    println!("cargo:rustc-env=DINGDA_LICENSE_TARGET_TRIPLE={license_triple}");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
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
    println!("cargo:rustc-env=DINGDA_LICENSE_VERIFIER_SHA256={sha}");
    println!("cargo:rustc-env=DINGDA_LICENSE_ATTEST_KEY_HEX={attest}");
}

/// Windows 上固定映射为 `*-windows-msvc`，与 bundled verifier 命名一致。
fn license_target_triple(target: &str) -> String {
    if cfg!(target_os = "windows") || target.contains("windows") {
        if target.contains("windows-gnu") {
            return target.replace("windows-gnu", "windows-msvc");
        }
        if target.contains("windows-msvc") {
            return target.to_string();
        }
        return "x86_64-pc-windows-msvc".into();
    }
    target.to_string()
}

fn read_trimmed(path: PathBuf) -> String {
    fs::read_to_string(&path)
        .map(|value| value.trim().to_string())
        .unwrap_or_default()
}
