//! 内置插件目录与本地安装状态。
//!
//! 每个插件落在 `{app_local_data}/plugins/{plugin_id}/` 下；OCR 语言包在
//! `plugins/ocr/tessdata/`。卸载只删除本应用 plugins 目录内的文件。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-19

use common::contracts::{PluginIpcListResponse, PluginItem};
use common::DingDaResult;
use std::path::{Path, PathBuf};

/// OCR 插件稳定 id。
pub const PLUGIN_ID_OCR: &str = "ocr";

/// 插件需下载的单个资源。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
#[derive(Debug, Clone, Copy)]
pub struct PluginAsset {
    /// 落盘文件名（如 `chi_sim.traineddata`）。
    pub file_name: &'static str,
    /// HTTP 下载地址。
    pub url: &'static str,
}

/// OCR 简体中文 + 英文语言包（GitHub tessdata，不随安装包分发）。
const OCR_ASSETS: &[PluginAsset] = &[
    PluginAsset {
        file_name: "eng.traineddata",
        url: "https://github.com/tesseract-ocr/tessdata/raw/main/eng.traineddata",
    },
    PluginAsset {
        file_name: "chi_sim.traineddata",
        url: "https://github.com/tesseract-ocr/tessdata/raw/main/chi_sim.traineddata",
    },
];

/// 插件根目录 `{plugins_dir}/{plugin_id}`。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
///
/// * `plugins_dir` — 本应用插件根
/// * `plugin_id` — 插件 id
///
/// # 返回值
///
/// 该插件专属文件夹。
pub fn plugin_root(plugins_dir: &Path, plugin_id: &str) -> PathBuf {
    plugins_dir.join(plugin_id)
}

/// 插件资源落盘目录（OCR 为 `plugins/ocr/tessdata`）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
///
/// * `plugins_dir` — 本应用插件根
/// * `plugin_id` — 插件 id
///
/// # 返回值
///
/// 资源文件目录。
pub fn plugin_install_dir(plugins_dir: &Path, plugin_id: &str) -> PathBuf {
    match plugin_id {
        PLUGIN_ID_OCR => plugin_root(plugins_dir, plugin_id).join("tessdata"),
        other => plugin_root(plugins_dir, other),
    }
}

/// 列出内置插件及其本地安装状态。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
///
/// * `plugins_dir` — 本应用插件根
/// * `legacy_ocr_dir` — 旧版 `{data}/tessdata`
///
/// # 返回值
///
/// 插件列表。
pub fn list_plugins(plugins_dir: &Path, legacy_ocr_dir: &Path) -> PluginIpcListResponse {
    PluginIpcListResponse {
        items: vec![ocr_item(plugins_dir, legacy_ocr_dir, None)],
    }
}

/// 返回指定插件的下载资源；未知 id 报错。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
///
/// * `plugin_id` — 插件 id
///
/// # 返回值
///
/// 资源列表；未知插件返回错误。
pub fn plugin_assets(plugin_id: &str) -> DingDaResult<&'static [PluginAsset]> {
    match plugin_id {
        PLUGIN_ID_OCR => Ok(OCR_ASSETS),
        other => Err(format!(
            "未知插件 id={other}；请从设置-插件列表选择已支持的插件（当前仅 ocr）"
        )
        .into()),
    }
}

/// 卸载指定插件：删除本应用 `plugins/{id}/` 整目录；OCR 同时清理旧版 `{data}/tessdata`。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
///
/// * `plugins_dir` — 本应用插件根
/// * `legacy_ocr_dir` — 旧版 tessdata 目录
/// * `plugin_id` — 插件 id
///
/// # 返回值
///
/// 卸载后的插件条目。
pub fn uninstall_plugin(
    plugins_dir: &Path,
    legacy_ocr_dir: &Path,
    plugin_id: &str,
) -> DingDaResult<PluginItem> {
    match plugin_id {
        PLUGIN_ID_OCR => uninstall_ocr(plugins_dir, legacy_ocr_dir),
        other => Err(format!(
            "未知插件 id={other}；请从设置-插件列表选择已支持的插件（当前仅 ocr）"
        )
        .into()),
    }
}

/// 构造 OCR 插件条目。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
///
/// * `plugins_dir` — 本应用插件根
/// * `legacy_ocr_dir` — 旧版 tessdata 目录
/// * `error` — 可选错误信息（失败态）
///
/// # 返回值
///
/// OCR 插件条目。
pub fn ocr_item(plugins_dir: &Path, legacy_ocr_dir: &Path, error: Option<String>) -> PluginItem {
    let installed = ocr_installed(plugins_dir, legacy_ocr_dir);
    let status = if error.is_some() {
        "failed"
    } else if installed {
        "installed"
    } else {
        "not_installed"
    };
    PluginItem {
        id: PLUGIN_ID_OCR.to_string(),
        name: "OCR".to_string(),
        description:
            "本地 Tesseract 文字识别（简体中文 + 英文语言模型），按需下载，不随安装包分发。"
                .to_string(),
        status: status.to_string(),
        error,
    }
}

/// 下载过程中的临时文件路径。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
///
/// * `dest` — 最终文件路径
///
/// # 返回值
///
/// `.part` 临时路径。
pub fn tmp_path(dest: &Path) -> PathBuf {
    dest.with_extension(format!(
        "{}.part",
        dest.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin")
    ))
}

fn ocr_installed(plugins_dir: &Path, legacy_ocr_dir: &Path) -> bool {
    let new_dir = plugin_install_dir(plugins_dir, PLUGIN_ID_OCR);
    assets_present(&new_dir) || assets_present(legacy_ocr_dir)
}

fn assets_present(dir: &Path) -> bool {
    OCR_ASSETS
        .iter()
        .all(|asset| dir.join(asset.file_name).is_file())
}

fn uninstall_ocr(plugins_dir: &Path, legacy_ocr_dir: &Path) -> DingDaResult<PluginItem> {
    let root = plugin_root(plugins_dir, PLUGIN_ID_OCR);
    let mut first_error: Option<String> = None;

    if root.exists() {
        if let Err(error) = std::fs::remove_dir_all(&root) {
            first_error = Some(format!(
                "删除插件目录失败 path={} reason={error}；请确认目录未被占用后重试",
                root.display()
            ));
        }
    }

    if legacy_ocr_dir.exists() {
        for asset in OCR_ASSETS {
            remove_asset_files(&legacy_ocr_dir.join(asset.file_name), &mut first_error);
        }
        let _ = std::fs::remove_dir(legacy_ocr_dir);
    }

    Ok(ocr_item(plugins_dir, legacy_ocr_dir, first_error))
}

fn remove_asset_files(path: &Path, first_error: &mut Option<String>) {
    if let Err(error) = std::fs::remove_file(path) {
        if error.kind() != std::io::ErrorKind::NotFound && first_error.is_none() {
            *first_error = Some(format!(
                "删除插件文件失败 path={} reason={error}；请确认文件未被占用后重试",
                path.display()
            ));
        }
    }
    let tmp = tmp_path(path);
    let _ = std::fs::remove_file(tmp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("dingda-plugin-test-{nanos}"));
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn list_not_installed_when_files_missing() {
        let root = temp_dir();
        let plugins_dir = root.join("plugins");
        let legacy = root.join("tessdata");
        let list = list_plugins(&plugins_dir, &legacy);
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].status, "not_installed");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn list_installed_under_plugin_folder() {
        let root = temp_dir();
        let plugins_dir = root.join("plugins");
        let install_dir = plugin_install_dir(&plugins_dir, PLUGIN_ID_OCR);
        fs::create_dir_all(&install_dir).expect("mkdir");
        for asset in OCR_ASSETS {
            fs::write(install_dir.join(asset.file_name), b"stub").expect("write");
        }
        let list = list_plugins(&plugins_dir, &root.join("tessdata"));
        assert_eq!(list.items[0].status, "installed");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn uninstall_removes_plugin_folder_only() {
        let root = temp_dir();
        let plugins_dir = root.join("plugins");
        let legacy = root.join("tessdata");
        let install_dir = plugin_install_dir(&plugins_dir, PLUGIN_ID_OCR);
        fs::create_dir_all(&install_dir).expect("mkdir");
        for asset in OCR_ASSETS {
            fs::write(install_dir.join(asset.file_name), b"stub").expect("write");
        }
        fs::write(root.join("other.txt"), b"keep").expect("write");

        uninstall_plugin(&plugins_dir, &legacy, PLUGIN_ID_OCR).expect("uninstall");

        assert!(!plugin_root(&plugins_dir, PLUGIN_ID_OCR).exists());
        assert!(root.join("other.txt").is_file());
        let after = list_plugins(&plugins_dir, &legacy);
        assert_eq!(after.items[0].status, "not_installed");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn assets_reject_unknown_plugin() {
        let error = plugin_assets("unknown").expect_err("unknown");
        assert!(error.to_string().contains("未知插件"));
    }
}
