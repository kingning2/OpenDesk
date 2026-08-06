//! 工具下载与安装：reqwest 异步流式下载 + 解压 / 静默安装。
//!
//! 全程不阻塞 UI：网络用 async client，解压 / 安装器执行放在 `spawn_blocking`。
//! 进度经回调推送（由调用方转发为 Tauri 事件）。解压时校验路径防止 zip-slip。

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::Client;

use super::detect::{executable_name, find_executable_recursive};
use super::urls::{download_kind, download_url, installer_default_dir, DownloadKind};
use super::{tool_install_dir, ToolId};

/// 下载过程中的进度事件。
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub status: &'static str,
    pub error_message: Option<String>,
}

/// 下载失败错误。
#[derive(Debug, Clone)]
pub enum DownloadError {
    /// 该平台无可用下载（如非 Windows 的 Tesseract）。
    NotAvailable(String),
    /// 网络 / 写入失败。
    Io(String),
    /// 归档解压失败或包含不安全路径。
    Extract(String),
    /// 安装器执行失败。
    Install(String),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::NotAvailable(message) => write!(f, "{message}"),
            DownloadError::Io(message) => write!(f, "download failed: {message}"),
            DownloadError::Extract(message) => write!(f, "extract failed: {message}"),
            DownloadError::Install(message) => write!(f, "install failed: {message}"),
        }
    }
}

impl std::error::Error for DownloadError {}

/// 下载临时文件名（扩展名与解压器分发对齐，避免 `.tmp` 无法匹配）。
fn archive_file_name(tool: ToolId) -> String {
    match download_kind(tool) {
        DownloadKind::Archive => match tool {
            ToolId::Pandoc => "download.zip".to_string(),
            ToolId::Pdfium => "download.tgz".to_string(),
            ToolId::Tesseract => "download.exe".to_string(),
        },
        DownloadKind::Installer => "download.exe".to_string(),
    }
}

/// 下载并安装一个工具。`progress` 回调在下载 / 解压各阶段触发。
pub async fn download_tool(
    tool: ToolId,
    mut progress: impl FnMut(DownloadProgress),
) -> Result<(), DownloadError> {
    let url = download_url(tool);
    if url.is_empty() {
        return Err(DownloadError::NotAvailable(format!(
            "{} 在此平台无可下载版本（建议用系统包管理器安装）",
            tool.display_name()
        )));
    }

    let install_dir = tool_install_dir(tool);
    std::fs::create_dir_all(&install_dir).map_err(|error| DownloadError::Io(error.to_string()))?;

    tracing::info!(
        target: "lifecycle",
        tool = tool.as_str(),
        url,
        target_dir = %install_dir.display(),
        "tool download started"
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|error| DownloadError::Io(error.to_string()))?;

    progress(DownloadProgress {
        bytes_downloaded: 0,
        bytes_total: 0,
        status: "downloading",
        error_message: None,
    });

    let mut response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| DownloadError::Io(error.to_string()))?;
    let total = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);

    // 下载到带正确扩展名的临时文件，避免中断污染安装目录。
    let archive_path = install_dir.join(archive_file_name(tool));
    let mut file =
        File::create(&archive_path).map_err(|error| DownloadError::Io(error.to_string()))?;
    let mut downloaded = 0u64;
    // 低频进度日志：每跨越 10% 或每 1MB 记一次，避免每次 chunk 刷屏。
    let mut last_bucket = u64::MAX;
    let mut last_mb = 0u64;
    loop {
        let read = response
            .chunk()
            .await
            .map_err(|error| DownloadError::Io(error.to_string()))?;
        let Some(chunk) = read else { break };
        file.write_all(&chunk)
            .map_err(|error| DownloadError::Io(error.to_string()))?;
        downloaded += chunk.len() as u64;
        progress(DownloadProgress {
            bytes_downloaded: downloaded,
            bytes_total: total,
            status: "downloading",
            error_message: None,
        });
        // 进度桶：有总长按百分比（每 10% 一档），否则按 MB（每 1MB 一档）。
        let bucket = if total > 0 {
            downloaded
                .saturating_mul(10)
                .checked_div(total)
                .unwrap_or(10)
        } else {
            downloaded / (1024 * 1024)
        };
        let mb = downloaded / (1024 * 1024);
        if bucket != last_bucket || mb > last_mb {
            tracing::debug!(
                tool = tool.as_str(),
                downloaded,
                total,
                "tool download progress"
            );
            last_bucket = bucket;
            last_mb = mb;
        }
    }
    drop(file);

    tracing::info!(
        target: "lifecycle",
        tool = tool.as_str(),
        downloaded,
        total,
        "tool download finished"
    );

    progress(DownloadProgress {
        bytes_downloaded: downloaded,
        bytes_total: total,
        status: "extracting",
        error_message: None,
    });

    // 解压 / 静默安装是 CPU + 子进程密集操作，放 blocking 线程池。
    let extract_archive_path = archive_path.clone();
    let extract_install_dir = install_dir.clone();
    let result = tokio::task::spawn_blocking(move || match download_kind(tool) {
        DownloadKind::Archive => extract_archive(&extract_archive_path, &extract_install_dir, tool),
        DownloadKind::Installer => run_installer(&extract_archive_path, &extract_install_dir, tool),
    })
    .await
    .map_err(|error| DownloadError::Io(error.to_string()))?;
    let _ = std::fs::remove_file(&archive_path);
    match &result {
        Ok(()) => tracing::info!(target: "lifecycle", tool = tool.as_str(), "tool installed"),
        Err(error) => {
            tracing::warn!(target: "lifecycle", tool = tool.as_str(), %error, "tool install failed")
        }
    }
    result
}

/// 解压归档到安装目录。
fn extract_archive(
    archive_path: &Path,
    install_dir: &Path,
    tool: ToolId,
) -> Result<(), DownloadError> {
    let extension = archive_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_lowercase();
    match extension.as_str() {
        "zip" => extract_zip(archive_path, install_dir, tool),
        "tgz" | "gz" => extract_tgz(archive_path, install_dir, tool),
        other => Err(DownloadError::Extract(format!(
            "unsupported archive type .{other}"
        ))),
    }
}

fn extract_zip(archive_path: &Path, install_dir: &Path, tool: ToolId) -> Result<(), DownloadError> {
    let file = File::open(archive_path).map_err(|error| DownloadError::Io(error.to_string()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|error| DownloadError::Extract(error.to_string()))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| DownloadError::Extract(error.to_string()))?;
        let entry_name = entry.name().to_string();
        let out_path = sanitized_path(install_dir, &entry_name)?;
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|error| DownloadError::Io(error.to_string()))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| DownloadError::Io(error.to_string()))?;
            }
            let mut out =
                File::create(&out_path).map_err(|error| DownloadError::Io(error.to_string()))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|error| DownloadError::Io(error.to_string()))?;
        }
    }
    drop(archive);
    validate_install(install_dir, tool)?;
    mark_installed(install_dir, tool);
    Ok(())
}

fn extract_tgz(archive_path: &Path, install_dir: &Path, tool: ToolId) -> Result<(), DownloadError> {
    let file = File::open(archive_path).map_err(|error| DownloadError::Io(error.to_string()))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    // 逐条解压并校验路径。
    let mut entries = archive
        .entries()
        .map_err(|error| DownloadError::Extract(error.to_string()))?;
    while let Some(mut entry) = entries
        .next()
        .transpose()
        .map_err(|error| DownloadError::Extract(error.to_string()))?
    {
        let entry_name = entry
            .path()
            .map_err(|error| DownloadError::Extract(error.to_string()))?
            .to_string_lossy()
            .to_string();
        let out_path = sanitized_path(install_dir, &entry_name)?;
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|error| DownloadError::Io(error.to_string()))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| DownloadError::Io(error.to_string()))?;
            }
            let mut out =
                File::create(&out_path).map_err(|error| DownloadError::Io(error.to_string()))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|error| DownloadError::Io(error.to_string()))?;
        }
    }
    validate_install(install_dir, tool)?;
    mark_installed(install_dir, tool);
    Ok(())
}

/// 把归档内路径解析为安装目录下的安全绝对路径；拒绝越界（zip-slip）。
fn sanitized_path(install_dir: &Path, entry_name: &str) -> Result<PathBuf, DownloadError> {
    let clean = entry_name
        .trim_start_matches('/')
        .trim_start_matches('\\')
        .replace('\\', "/");
    let candidate = install_dir.join(&clean);
    if !candidate.starts_with(install_dir) {
        return Err(DownloadError::Extract(format!(
            "unsafe archive path: {entry_name}"
        )));
    }
    Ok(candidate)
}

/// 运行 NSIS 安装器（tesseract）到工具目录，静默安装。
///
/// 嵌套 NSIS 安装器可能忽略 `/D=` 把产物装到默认位置，因此 exit 0 不代表装进
/// 了工具目录：先做真实校验，失败则从默认安装位置回收，仍失败才报错。
fn run_installer(
    installer_path: &Path,
    install_dir: &Path,
    tool: ToolId,
) -> Result<(), DownloadError> {
    let status = std::process::Command::new(installer_path)
        .arg("/S")
        .arg(format!("/D={}", install_dir.display()))
        .status()
        .map_err(|error| DownloadError::Install(error.to_string()))?;
    if !status.success() {
        return Err(DownloadError::Install(format!(
            "{} installer exited with {status}",
            tool.display_name()
        )));
    }
    if let Ok(()) = validate_install(install_dir, tool) {
        mark_installed(install_dir, tool);
        return Ok(());
    }
    recover_install(install_dir, tool)?;
    mark_installed(install_dir, tool);
    Ok(())
}

/// 校验工具是否真的装进了安装目录（存在可执行文件/动态库）。
///
/// 可复用：任何工具在下载/安装/回收后都可用它确认产物真实落地，避免
/// 「exit 0 + 标记文件」造成的假成功。
fn validate_install(install_dir: &Path, tool: ToolId) -> Result<(), DownloadError> {
    let name = executable_name(tool);
    let direct = install_dir.join(name);
    if direct.is_file() || find_executable_recursive(install_dir, name).is_some() {
        return Ok(());
    }
    Err(DownloadError::Install(format!(
        "{} 未找到可执行文件 {}（安装可能落到了其他目录）",
        tool.display_name(),
        name
    )))
}

/// 从安装器的默认安装位置回收产物到工具目录。
///
/// 可复用：安装器型工具（如 Tesseract）静默安装可能忽略 `/D=`，把文件装到
/// `installer_default_dir` 指定的系统目录。此函数检测到后整体拷贝回工具目录，
/// 归一化工具目录为唯一事实源（检测逻辑无需改动）。
fn recover_install(install_dir: &Path, tool: ToolId) -> Result<(), DownloadError> {
    let Some(default_dir) = installer_default_dir(tool) else {
        return validate_install(install_dir, tool);
    };
    let name = executable_name(tool);
    let source = default_dir.join(name);
    if !source.is_file() {
        return Err(DownloadError::Install(format!(
            "{} 安装校验失败且默认安装目录 {} 也没有产物",
            tool.display_name(),
            default_dir.display()
        )));
    }
    tracing::info!(
        target: "lifecycle",
        tool = tool.as_str(),
        from = %default_dir.display(),
        to = %install_dir.display(),
        "recovering tool install from default location"
    );
    copy_dir_recursive(&default_dir, install_dir).map_err(|error| {
        DownloadError::Install(format!(
            "从 {} 回收 {} 失败: {error}",
            default_dir.display(),
            tool.display_name()
        ))
    })?;
    validate_install(install_dir, tool)
}

/// 递归拷贝目录内容到目标目录（合并，不删除目标已有文件）。
fn copy_dir_recursive(source: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dest = target.join(&file_name);
        if path.is_dir() {
            copy_dir_recursive(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

/// 写入 `.installed` 标记文件，供检测模块判断已安装。
fn mark_installed(install_dir: &Path, tool: ToolId) {
    let _ = std::fs::write(install_dir.join(".installed"), tool.as_str());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_file_name_matches_extractor() {
        // 临时文件扩展名必须能被 `extract_archive` 的扩展名分发正确匹配。
        assert_eq!(archive_file_name(ToolId::Pandoc), "download.zip");
        assert_eq!(archive_file_name(ToolId::Pdfium), "download.tgz");
        assert_eq!(archive_file_name(ToolId::Tesseract), "download.exe");
    }
}
