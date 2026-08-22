// 编译期渠道平台 cfg 注入 — 供各 crate 的 `build.rs` 通过 `include!` 复用。
//
// 优先级：
// 1. Cargo feature：`xianyu` / `ali1688`（`CARGO_FEATURE_*`）
// 2. 环境变量 `DINGDA_CHANNEL_PLATFORMS`（逗号分隔，`1688` 视为 `ali1688`）
// 3. 环境变量 `DINGDA_CHANNEL_PLATFORM`（单站，兼容旧用法）
// 4. `channel-platforms.json` 的 `default`
//
// 输出：
// - `cargo:rustc-env=DINGDA_CHANNEL_PLATFORM=<主站>`
// - `cargo:rustc-env=DINGDA_CHANNEL_PLATFORMS=<id,id>`
// - `cargo:rustc-cfg=platform_<id>`（可多个）
//
// 作者：Xiaoman
// 创建时间：2026-08-18

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct ChannelPlatformsConfig {
    default: String,
    platforms: Vec<String>,
}

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
    println!("cargo:rerun-if-env-changed=DINGDA_CHANNEL_PLATFORM");
    println!("cargo:rerun-if-env-changed=DINGDA_CHANNEL_PLATFORMS");

    ChannelPlatformsConfig { default, platforms }
}

fn cfg_key(platform_id: &str) -> String {
    format!("platform_{}", platform_id.replace('-', "_"))
}

fn canonicalize_id(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1688" | "ali1688" => "ali1688".to_string(),
        other => other.to_string(),
    }
}

fn from_cargo_features() -> Vec<String> {
    let mut enabled = Vec::new();
    if env::var("CARGO_FEATURE_XIANYU").is_ok() {
        enabled.push("xianyu".to_string());
    }
    if env::var("CARGO_FEATURE_ALI1688").is_ok() {
        enabled.push("ali1688".to_string());
    }
    enabled
}

fn parse_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(canonicalize_id)
        .filter(|item| !item.is_empty())
        .collect()
}

fn validate(ids: &[String], known: &[String]) -> Vec<String> {
    let mut unique = BTreeSet::new();
    for id in ids {
        if !known.iter().any(|item| item == id) {
            panic!("未知渠道平台 {id:?}，可选: {}", known.join(", "));
        }
        unique.insert(id.clone());
    }
    unique.into_iter().collect()
}

fn primary_platform(enabled: &[String]) -> String {
    if enabled.iter().any(|id| id == "xianyu") {
        return "xianyu".to_string();
    }
    enabled.first().cloned().expect("至少启用一个渠道平台")
}

pub fn emit_channel_platform_cfg(config_rel_to_manifest: &str) {
    let config = load_config(config_rel_to_manifest);
    for id in &config.platforms {
        println!("cargo:rustc-check-cfg=cfg({})", cfg_key(id));
    }

    let enabled = {
        let from_features = from_cargo_features();
        if !from_features.is_empty() {
            validate(&from_features, &config.platforms)
        } else if let Ok(list) = env::var("DINGDA_CHANNEL_PLATFORMS") {
            validate(&parse_list(&list), &config.platforms)
        } else if let Ok(single) = env::var("DINGDA_CHANNEL_PLATFORM") {
            validate(&[canonicalize_id(&single)], &config.platforms)
        } else {
            validate(&[canonicalize_id(&config.default)], &config.platforms)
        }
    };

    let primary = primary_platform(&enabled);
    let joined = enabled.join(",");
    println!("cargo:rustc-env=DINGDA_CHANNEL_PLATFORM={primary}");
    println!("cargo:rustc-env=DINGDA_CHANNEL_PLATFORMS={joined}");
    for id in &enabled {
        println!("cargo:rustc-cfg={}", cfg_key(id));
    }
}
