//! 各平台解析工具下载 URL。
//!
//! 版本号与 URL 集中在此，可被同名环境变量覆盖（便于内网镜像或固定版本）。

use super::ToolId;

/// Pandoc 版本（GitHub release tag）。
const PANDOC_VERSION: &str = "3.6.4";
/// Tesseract 版本（UB-Mannheim 打包，Windows 安装器）。
const TESSERACT_VERSION: &str = "5.5.3.20260724";
/// PDFium 版本（bblanchon/PDFium-binaries release tag）。
const PDFIUM_VERSION: &str = "7381";

/// 环境变量覆盖：`KNOWLEDGE_PANDOC_URL` / `KNOWLEDGE_TESSERACT_URL` / `KNOWLEDGE_PDFIUM_URL`。
fn env_url(var: &str, default: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| default.to_string())
}

/// 下载类型：解压型（zip/tgz，解压即用）或安装器型（exe，静默安装）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadKind {
    /// zip / tar.gz 归档，解压到安装目录。
    Archive,
    /// 可执行安装器，需以静默参数运行。
    Installer,
}

/// 指定工具的下载类型。
pub fn download_kind(tool: ToolId) -> DownloadKind {
    match tool {
        ToolId::Pandoc | ToolId::Pdfium => DownloadKind::Archive,
        ToolId::Tesseract => DownloadKind::Installer,
    }
}

/// 返回指定工具的下载 URL（按当前平台）。
///
/// 非 Windows 平台的 Tesseract 无官方可下载二进制（建议系统包管理器），返回空串表示不可下载。
pub fn download_url(tool: ToolId) -> String {
    match tool {
        ToolId::Pandoc => env_url("KNOWLEDGE_PANDOC_URL", &pandoc_url()),
        ToolId::Tesseract => env_url("KNOWLEDGE_TESSERACT_URL", &tesseract_url()),
        ToolId::Pdfium => env_url("KNOWLEDGE_PDFIUM_URL", &pdfium_url()),
    }
}

fn pandoc_url() -> String {
    // pandoc GitHub release 按平台打包。
    #[cfg(target_os = "windows")]
    let file = format!("pandoc-{PANDOC_VERSION}-windows-x86_64.zip");
    #[cfg(target_os = "linux")]
    let file = format!("pandoc-{PANDOC_VERSION}-linux-amd64.tar.gz");
    #[cfg(target_os = "macos")]
    let file = format!("pandoc-{PANDOC_VERSION}-macOS-x86_64.zip");
    format!("https://github.com/jgm/pandoc/releases/download/{PANDOC_VERSION}/{file}")
}

fn tesseract_url() -> String {
    // UB-Mannheim 提供含 eng/chi_sim/chi_tra tessdata 的 Windows 安装器（NSIS，支持 /S 静默）。
    #[cfg(target_os = "windows")]
    let url = format!(
        "https://digi.bib.uni-mannheim.de/tesseract/tesseract-ocr-w64-setup-{TESSERACT_VERSION}.exe"
    );
    #[cfg(not(target_os = "windows"))]
    let url = String::new();
    url
}

fn pdfium_url() -> String {
    // bblanchon/PDFium-binaries：按平台下载，含 pdfium.dll / libpdfium.so。
    let platform = if cfg!(target_os = "windows") {
        "win-x64"
    } else if cfg!(target_os = "linux") {
        "linux-x64"
    } else {
        "mac-x64"
    };
    format!(
        "https://github.com/bblanchon/PDFium-binaries/releases/download/chromium%2F{PDFIUM_VERSION}/pdfium-{platform}.tgz"
    )
}
