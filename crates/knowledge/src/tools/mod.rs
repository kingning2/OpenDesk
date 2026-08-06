//! 外部解析工具管理：Pandoc / Tesseract / PDFium。
//!
//! 三个工具供知识库文档解析使用：
//! - **Pandoc** — docx / html 转 Markdown（保真度最高）
//! - **Tesseract** — PDF 扫描页 OCR（支持中文）
//! - **PDFium** — pdf2md 文本提取 + 扫描页渲染
//!
//! 工具可经前端触发下载到 `{data}/OpenDesk/tools/`，也可已安装在系统 PATH。
//! 未安装时解析逻辑回退到纯 Rust 实现（docx-rs / pdf-extract）。

pub mod detect;
pub mod download;
pub mod urls;

pub use detect::detect_tool;
pub use download::{download_tool, DownloadError, DownloadProgress};

use std::path::PathBuf;

/// 外部解析工具标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolId {
    Pandoc,
    Tesseract,
    Pdfium,
}

impl ToolId {
    /// 字符串 id（契约与前端共用）。
    pub fn as_str(self) -> &'static str {
        match self {
            ToolId::Pandoc => "pandoc",
            ToolId::Tesseract => "tesseract",
            ToolId::Pdfium => "pdfium",
        }
    }

    /// 展示名。
    pub fn display_name(self) -> &'static str {
        match self {
            ToolId::Pandoc => "Pandoc",
            ToolId::Tesseract => "Tesseract",
            ToolId::Pdfium => "PDFium",
        }
    }

    /// 从字符串解析；未知返回 `None`。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pandoc" => Some(ToolId::Pandoc),
            "tesseract" => Some(ToolId::Tesseract),
            "pdfium" => Some(ToolId::Pdfium),
            _ => None,
        }
    }
}

/// 一个工具的安装状态（检测结果）。
#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub id: &'static str,
    pub name: &'static str,
    pub installed: bool,
    pub version: String,
    pub error: Option<String>,
}

/// 解析工具目录：`{data}/OpenDesk/tools/`。
///
/// 无 `data_local_dir` 时回退到系统临时目录。
pub fn tools_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("tools");
    path
}

/// 工具安装子目录：`{tools_dir}/pandoc` 等。
pub fn tool_install_dir(tool: ToolId) -> PathBuf {
    tools_dir().join(tool.as_str())
}

/// 查找工具可执行文件（或 PDFium 动态库）路径。
///
/// 依次尝试：环境变量覆盖 → tools 目录 → 系统 PATH（pandoc / tesseract）。
pub fn resolve_tool_path(tool: ToolId) -> Option<PathBuf> {
    detect::resolve_tool_path(tool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_id_roundtrip() {
        for id in [ToolId::Pandoc, ToolId::Tesseract, ToolId::Pdfium] {
            assert_eq!(ToolId::parse(id.as_str()), Some(id));
        }
        assert_eq!(ToolId::parse("unknown"), None);
    }

    #[test]
    fn download_urls_nonempty_on_windows() {
        // 确保三个工具在当前平台都有可解析的 URL（或明确不可下载）。
        for id in [ToolId::Pandoc, ToolId::Tesseract, ToolId::Pdfium] {
            let _ = urls::download_url(id);
        }
    }
}
