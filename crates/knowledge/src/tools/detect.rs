//! 工具检测：判断是否已安装、解析可执行文件路径、提取版本号。

use std::path::{Path, PathBuf};
use std::process::Command;

use super::{tool_install_dir, ToolId, ToolStatus};

/// 环境变量覆盖：`KNOWLEDGE_PANDOC_PATH` / `KNOWLEDGE_TESSERACT_PATH` / `KNOWLEDGE_PDFIUM_PATH`。
fn env_override(tool: ToolId) -> Option<PathBuf> {
    let var = match tool {
        ToolId::Pandoc => "KNOWLEDGE_PANDOC_PATH",
        ToolId::Tesseract => "KNOWLEDGE_TESSERACT_PATH",
        ToolId::Pdfium => "KNOWLEDGE_PDFIUM_PATH",
    };
    std::env::var_os(var).map(PathBuf::from)
}

/// 工具安装目录下的可执行文件名（PDFium 为动态库）。
fn executable_name(tool: ToolId) -> &'static str {
    match tool {
        ToolId::Pandoc => {
            if cfg!(target_os = "windows") {
                "pandoc.exe"
            } else {
                "pandoc"
            }
        }
        ToolId::Tesseract => {
            if cfg!(target_os = "windows") {
                "tesseract.exe"
            } else {
                "tesseract"
            }
        }
        ToolId::Pdfium => {
            if cfg!(target_os = "windows") {
                "pdfium.dll"
            } else if cfg!(target_os = "macos") {
                "libpdfium.dylib"
            } else {
                "libpdfium.so"
            }
        }
    }
}

/// 解析工具可执行文件路径：env 覆盖 → 安装目录 → 系统 PATH（pandoc/tesseract）。
pub fn resolve_tool_path(tool: ToolId) -> Option<PathBuf> {
    if let Some(path) = env_override(tool) {
        if path.is_file() {
            return Some(path);
        }
    }
    let install_dir = tool_install_dir(tool);
    let installed = install_dir.join(".installed");
    let local = install_dir.join(executable_name(tool));
    if installed.is_file() && local.is_file() {
        return Some(local);
    }
    // 递归查找归档常见层级（pandoc-3.6.4/pandoc.exe 等）。
    if installed.is_file() {
        if let Some(found) = find_executable_recursive(&install_dir, executable_name(tool)) {
            return Some(found);
        }
    }
    // 系统 PATH。
    match tool {
        ToolId::Pandoc | ToolId::Tesseract => find_on_path(executable_name(tool)),
        ToolId::Pdfium => None,
    }
}

/// 检测工具安装状态与版本。
pub fn detect_tool(tool: ToolId) -> ToolStatus {
    match resolve_tool_path(tool) {
        Some(path) => {
            let (version, error) = read_version(&path);
            ToolStatus {
                id: tool.as_str(),
                name: tool.display_name(),
                installed: error.is_none(),
                version,
                error,
            }
        }
        None => ToolStatus {
            id: tool.as_str(),
            name: tool.display_name(),
            installed: false,
            version: String::new(),
            error: None,
        },
    }
}

/// 执行 `--version` 提取版本号；失败返回错误描述。
fn read_version(path: &Path) -> (String, Option<String>) {
    let result = Command::new(path)
        .arg("--version")
        .output()
        .map_err(|error| error.to_string());
    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = format!("{stdout}\n{stderr}");
            let version = first_line(&combined).unwrap_or_default();
            if output.status.success() {
                (version, None)
            } else {
                (String::new(), Some(version))
            }
        }
        Err(error) => (String::new(), Some(error)),
    }
}

fn first_line(text: &str) -> Option<String> {
    text.lines().next().map(str::to_string)
}

/// 在 PATH 中查找可执行文件。
fn find_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// 递归查找指定文件名（有限深度）。
fn find_executable_recursive(dir: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    let mut depth = 0usize;
    while let Some(current) = stack.pop() {
        if depth > 6 {
            break;
        }
        depth += 1;
        let entries = std::fs::read_dir(&current).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                    return Some(path);
                }
            } else if path.is_dir() {
                stack.push(path);
            }
        }
    }
    None
}
