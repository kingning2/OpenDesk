//! PDF 扫描页 OCR：pdfium-render 渲染页为 PNG，再调用系统 tesseract 识别。
//!
//! 依赖两个外部工具：PDFium（渲染）与 Tesseract（识别，需 eng+chi_sim tessdata）。
//! 任一缺失时返回 [`OcrError::ToolMissing`]，由调用方决定回退或报错。
//!
//! 全程异步：PDFium 渲染是 CPU 密集，放 `spawn_blocking`；tesseract 用 `tokio::process`。

use std::path::{Path, PathBuf};

use tokio::process::Command;

use common::tools::{detect::resolve_tool_path, ToolId};

/// OCR 失败错误。
#[derive(Debug, Clone)]
pub enum OcrError {
    /// 所需工具（PDFium / Tesseract）未安装。
    ToolMissing(String),
    /// PDFium 绑定 / 页面渲染失败。
    Render(String),
    /// Tesseract 执行失败。
    OcrFailed(String),
}

impl std::fmt::Display for OcrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OcrError::ToolMissing(message) => write!(f, "{message}"),
            OcrError::Render(message) => write!(f, "render failed: {message}"),
            OcrError::OcrFailed(message) => write!(f, "ocr failed: {message}"),
        }
    }
}

impl std::error::Error for OcrError {}

/// 使用 PDFium 渲染 PDF 的指定页为 PNG，返回临时 PNG 路径。
///
/// `pdf_path` 为已写盘的临时 PDF；`page_index` 从 0 开始。
/// 返回的 PNG 文件由调用方清理。渲染在 `spawn_blocking` 中执行，不阻塞 UI。
pub async fn render_pdf_page(
    pdf_path: &Path,
    page_index: i32,
    out_dir: &Path,
) -> Result<PathBuf, OcrError> {
    let pdfium_path = resolve_tool_path(ToolId::Pdfium)
        .ok_or_else(|| OcrError::ToolMissing("PDFium 未安装，无法渲染扫描页".into()))?;
    // pdfium-render 按名称加载库（Windows 搜 PATH / exe 目录），先把工具目录并入 PATH。
    bootstrap_pdfium_path(&pdfium_path);

    let pdf_path = pdf_path.to_path_buf();
    let out_dir = out_dir.to_path_buf();
    tokio::task::spawn_blocking(move || render_page_sync(&pdf_path, page_index, &out_dir))
        .await
        .map_err(|error| OcrError::Render(format!("render task join: {error}")))?
}

/// pdfium 渲染的同步实现（在 `spawn_blocking` 内调用）。
fn render_page_sync(pdf_path: &Path, page_index: i32, out_dir: &Path) -> Result<PathBuf, OcrError> {
    // 重新绑定：pdfium-render 只允许绑定一次，绑定失败会让 pdf2md 也失效，这里尝试绑定。
    let bindings = pdfium_render::prelude::Pdfium::bind_to_system_library()
        .map_err(|error| OcrError::Render(format!("pdfium bind: {error}")))?;
    let pdfium = pdfium_render::prelude::Pdfium::new(bindings);
    let document = pdfium
        .load_pdf_from_file(pdf_path, None)
        .map_err(|error| OcrError::Render(format!("open pdf: {error}")))?;
    let page = document
        .pages()
        .get(page_index)
        .map_err(|error| OcrError::Render(format!("get page {page_index}: {error}")))?;
    let width = page.width().value as i32;
    let height = page.height().value as i32;
    // 2x 渲染提高 OCR 清晰度。
    let bitmap = page
        .render(width * 2, height * 2, None)
        .map_err(|error| OcrError::Render(format!("render page: {error}")))?;
    let image = bitmap
        .as_image()
        .map_err(|error| OcrError::Render(format!("bitmap to image: {error}")))?;
    let out_path = out_dir.join(format!("page-{page_index}.png"));
    image
        .save(&out_path)
        .map_err(|error| OcrError::Render(format!("save png: {error}")))?;
    Ok(out_path)
}

/// 对 PNG 图片执行 Tesseract OCR，返回识别文本。
///
/// 语言为 `chi_sim+eng`（中文优先，英文兜底）；缺失 chi_sim 时回退 eng。
/// 子进程用 `tokio::process::Command`，不阻塞 UI。
pub async fn ocr_png(png_path: &Path) -> Result<String, OcrError> {
    let tesseract = resolve_tool_path(ToolId::Tesseract)
        .ok_or_else(|| OcrError::ToolMissing("Tesseract 未安装，无法识别扫描页".into()))?;
    let languages = ["chi_sim+eng", "eng"];
    let mut last_error: Option<String> = None;
    for langs in languages {
        let output = Command::new(&tesseract)
            .arg(png_path)
            .arg("stdout")
            .arg("-l")
            .arg(langs)
            .output()
            .await
            .map_err(|error| OcrError::OcrFailed(error.to_string()))?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(text);
        }
        last_error = Some(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Err(OcrError::OcrFailed(
        last_error.unwrap_or_else(|| "tesseract failed".into()),
    ))
}

/// 把 PDFium 所在目录并入 `PATH`，使 pdfium-render 能按名加载动态库。
///
/// pdf2md 内部使用 `bind_to_system_library()` 按名加载，且一旦失败会缓存错误；
/// 因此在任何 pdfium 调用前必须先把工具目录并入 PATH。
pub fn bootstrap_pdfium_path(pdfium_path: &Path) {
    if let Some(dir) = pdfium_path.parent() {
        let current = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = std::env::split_paths(&current).collect::<Vec<_>>();
        if !paths.contains(&dir.to_path_buf()) {
            paths.insert(0, dir.to_path_buf());
        }
        let joined = std::env::join_paths(paths).unwrap_or(current);
        // 进程级修改，仅影响本进程后续动态库加载。
        unsafe { std::env::set_var("PATH", joined) };
    }
}
