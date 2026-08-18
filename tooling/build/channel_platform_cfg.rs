// 编译期渠道平台 cfg 注入 — 供各 crate 的 `build.rs` 通过 `include!` 复用。
//
// 读取环境变量 `OPENDESK_CHANNEL_PLATFORM`（缺省见 `tooling/config/channel-platforms.json`），
// 校验后向 Cargo 输出：
// - `cargo:rustc-env=OPENDESK_CHANNEL_PLATFORM=<id>`
// - `cargo:rustc-cfg=platform_<id>`（如 `platform_xianyu`）
//
// 作者：Xiaoman
// 创建时间：2026-08-18

use std::env;
use std::fs;
use std::path::PathBuf;

// 渠道平台配置（与 `tooling/config/channel-platforms.json` 对齐）。
#[derive(Debug)]
struct ChannelPlatformsConfig {
    default: String,
    platforms: Vec<String>,
}

// 读取并校验渠道平台配置 JSON。
//
// # 参数
// - `config_path` — 相对 `CARGO_MANIFEST_DIR` 的配置文件路径
fn load_config(config_path: &str) -> ChannelPlatformsConfig {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join(config_path);
    let raw = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("读取渠道平台配置失败 {}: {error}", path.display());
    });
    let value: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|error| {
        panic!("解析渠道平台配置 JSON 失败 {}: {error}", path.display());
    });

    let default = value
        .get("default")
        .and_then(|item| item.as_str())
        .unwrap_or("xianyu")
        .to_string();

    let platforms = value
        .get("platforms")
        .and_then(|item| item.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(|id| id.as_str()))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if platforms.is_empty() {
        panic!("channel-platforms.json 未声明任何平台");
    }

    println!("cargo:rerun-if-changed={}", path.display());
    println!("cargo:rerun-if-env-changed=OPENDESK_CHANNEL_PLATFORM");

    ChannelPlatformsConfig { default, platforms }
}

// 将平台 id 转为 `platform_<id>` cfg 键（`-` → `_`）。
fn cfg_key(platform_id: &str) -> String {
    format!("platform_{}", platform_id.replace('-', "_"))
}

// 注入编译期渠道平台 cfg 与环境变量。
//
// # 参数
// - `config_rel_to_manifest` — 相对当前 crate manifest 的配置路径
pub fn emit_channel_platform_cfg(config_rel_to_manifest: &str) {
    let config = load_config(config_rel_to_manifest);
    for id in &config.platforms {
        println!("cargo:rustc-check-cfg=cfg({})", cfg_key(id));
    }
    let requested = env::var("OPENDESK_CHANNEL_PLATFORM").unwrap_or(config.default);

    if !config.platforms.iter().any(|id| id == &requested) {
        panic!(
            "未知 OPENDESK_CHANNEL_PLATFORM={requested:?}，可选: {}",
            config.platforms.join(", ")
        );
    }

    println!("cargo:rustc-env=OPENDESK_CHANNEL_PLATFORM={requested}");
    println!("cargo:rustc-cfg={}", cfg_key(&requested));
}
