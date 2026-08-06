//! 把上传文件的原始字节解析为 Markdown 文本。
//!
//! 按文件名扩展名分发：
//! - **pdf** — 逐页处理：文字页用 `pdf2md`（保留标题/表格结构，需 PDFium）；图片
//!   （扫描）页用 PDFium 渲染 + Tesseract OCR；均不可用时回退 `pdf-extract` 纯文本。
//! - **docx / html** — 优先用 Pandoc（`-t gfm`）转换；未安装时回退 docx-rs / html2md。
//! - **txt / md** — UTF-8 直通。
//!
//! 解析失败返回语义化错误，不 panic。

use std::path::Path;

use common::tools::{detect_tool, resolve_tool_path, ToolId};

/// 文档解析错误。
#[derive(Debug, Clone)]
pub enum ParseError {
    /// 不支持的扩展名。
    UnsupportedType(String),
    /// PDF 无文本层且 OCR 不可用（未装 Tesseract / PDFium）。
    PdfNoText(String),
    /// 解析失败。
    ParseFailed(String),
    /// 文本非 UTF-8。
    NotUtf8,
    /// 文件为空。
    Empty,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnsupportedType(ext) => write!(f, "unsupported file type: .{ext}"),
            ParseError::PdfNoText(message) => write!(f, "{message}"),
            ParseError::ParseFailed(message) => write!(f, "parse failed: {message}"),
            ParseError::NotUtf8 => write!(f, "text is not valid UTF-8"),
            ParseError::Empty => write!(f, "file is empty"),
        }
    }
}

impl std::error::Error for ParseError {}

/// 文件扩展名对应的来源类型标识（持久化到 `knowledge_doc.source_type`）。
pub fn source_type(name: &str) -> Option<&'static str> {
    let ext = Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_lowercase);
    match ext.as_deref() {
        Some("pdf") => Some("pdf"),
        Some("docx") => Some("docx"),
        Some("html") | Some("htm") => Some("html"),
        Some("txt") => Some("txt"),
        Some("md") | Some("markdown") => Some("md"),
        _ => None,
    }
}

/// 把文件字节解析为 Markdown 文本。
///
/// 异步实现：子进程（pandoc / tesseract）用 `tokio::process`，PDFium / pdf2md 推理
/// 在内部 `spawn_blocking` 执行，均不阻塞调用线程。
///
/// # 参数
/// - `name` — 原始文件名（含扩展名，决定解析器）
/// - `bytes` — 文件原始字节
///
/// # Errors
///
/// 扩展名不受支持、解析失败或无文本层时返回 [`ParseError`]。
pub async fn parse_to_markdown(name: &str, bytes: &[u8]) -> Result<String, ParseError> {
    if bytes.is_empty() {
        return Err(ParseError::Empty);
    }
    let ext = Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_lowercase)
        .ok_or_else(|| ParseError::UnsupportedType("".into()))?;
    match ext.as_str() {
        "pdf" => parse_pdf(bytes).await,
        "docx" => parse_docx(bytes).await,
        "html" | "htm" => parse_html(bytes).await,
        "txt" | "md" | "markdown" => parse_text(bytes),
        _ => Err(ParseError::UnsupportedType(ext)),
    }
}

/// PDF 解析：逐页分派「文字页 → pdf2md/pdf-extract；图片页 → OCR」。
///
/// 优先走 pdf2md（PDFium 可用时，保留标题/表格结构）；否则逐页用 pdf-extract 判断
/// 是否有文本层，空文本页尝试 PDFium 渲染 + Tesseract OCR；两者都缺则报错。
async fn parse_pdf(bytes: &[u8]) -> Result<String, ParseError> {
    // 优先路径：pdf2md 一次输出整篇 Markdown（需 PDFium）。
    if let Some(markdown) = parse_pdf_via_pdf2md(bytes).await {
        return Ok(markdown);
    }

    // 回退路径：逐页判断文本层，图片页走 OCR。
    let pages = pdf_extract::extract_text_from_mem_by_pages(bytes)
        .map_err(|error| ParseError::ParseFailed(error.to_string()))?;
    if pages.is_empty() {
        return Err(ParseError::PdfNoText(
            "PDF 无法提取文本，且无可用解析工具".into(),
        ));
    }
    let pdfium_ok = resolve_tool_path(ToolId::Pdfium).is_some();
    let tesseract_ok = resolve_tool_path(ToolId::Tesseract).is_some();

    let mut out = String::new();
    let mut ocr_needed = false;
    for (index, page_text) in pages.iter().enumerate() {
        if page_text.trim().is_empty() {
            // 图片页：渲染 + OCR。
            if pdfium_ok && tesseract_ok {
                ocr_needed = true;
                if let Ok(text) = ocr_page(bytes, index).await {
                    out.push_str(&text);
                    out.push_str("\n\n");
                }
            }
        } else {
            out.push_str(page_text);
            out.push_str("\n\n");
        }
    }
    let text = out.trim();
    if text.is_empty() {
        if ocr_needed {
            return Err(ParseError::PdfNoText(
                "PDF 为纯扫描件，但 OCR 未产出文本（请确认已安装 Tesseract 与 PDFium）".into(),
            ));
        }
        return Err(ParseError::PdfNoText(
            "PDF 无文本层，且无可用解析工具".into(),
        ));
    }
    Ok(text.to_string())
}

/// 用 pdf2md 提取整篇 Markdown；PDFium 不可用或提取失败时返回 `None`。
async fn parse_pdf_via_pdf2md(bytes: &[u8]) -> Option<String> {
    let pdfium_path = resolve_tool_path(ToolId::Pdfium)?;
    crate::ocr::bootstrap_pdfium_path(&pdfium_path);

    // pdf2md 需要文件路径；写入临时文件。
    let dir = tempfile::tempdir().ok()?;
    let input = dir.path().join("input.pdf");
    std::fs::write(&input, bytes).ok()?;
    let result = pdf2md::extract(&input).await.ok()?.markdown;
    let trimmed = result.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

/// 渲染 PDF 单页并 OCR，返回识别文本；失败时返回错误（调用方容忍）。
async fn ocr_page(bytes: &[u8], page_index: usize) -> Result<String, ParseError> {
    let dir = tempfile::tempdir().map_err(|error| ParseError::ParseFailed(error.to_string()))?;
    let pdf_path = dir.path().join("page.pdf");
    std::fs::write(&pdf_path, bytes).map_err(|error| ParseError::ParseFailed(error.to_string()))?;
    let png = crate::ocr::render_pdf_page(&pdf_path, page_index as i32, dir.path())
        .await
        .map_err(|error| ParseError::ParseFailed(error.to_string()))?;
    let text = crate::ocr::ocr_png(&png)
        .await
        .map_err(|error| ParseError::ParseFailed(error.to_string()))?;
    Ok(text)
}

/// 用 Pandoc 把文件转 GFM Markdown；Pandoc 未安装或失败时返回 `None`。
///
/// 子进程用 `tokio::process::Command`，文件读写用 `tokio::fs`，不阻塞调用线程。
async fn parse_via_pandoc(bytes: &[u8], extension: &str) -> Option<String> {
    if detect_tool(ToolId::Pandoc).installed {
        let pandoc = resolve_tool_path(ToolId::Pandoc)?;
        let dir = tempfile::tempdir().ok()?;
        let input = dir.path().join(format!("input.{extension}"));
        let output = dir.path().join("output.md");
        std::fs::write(&input, bytes).ok()?;
        let result = tokio::process::Command::new(&pandoc)
            .arg(&input)
            .arg("-t")
            .arg("gfm")
            .arg("-o")
            .arg(&output)
            .output()
            .await
            .ok()?;
        if result.status.success() {
            if let Ok(text) = std::fs::read_to_string(&output) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

async fn parse_docx(bytes: &[u8]) -> Result<String, ParseError> {
    // 优先 Pandoc（保留列表/富文本/表格结构）。
    if let Some(markdown) = parse_via_pandoc(bytes, "docx").await {
        return Ok(markdown);
    }
    // 回退：docx-rs 手动重建 Markdown（XML 解析是 CPU 密集，放 blocking 线程池）。
    let bytes = bytes.to_vec();
    let result = tokio::task::spawn_blocking(move || parse_docx_fallback(&bytes)).await;
    match result {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(error)) => Err(error),
        Err(error) => Err(ParseError::ParseFailed(format!(
            "docx parse task join: {error}"
        ))),
    }
}

/// docx-rs 回退解析的同步实现（在 `spawn_blocking` 内调用）。
fn parse_docx_fallback(bytes: &[u8]) -> Result<String, ParseError> {
    use docx_rs::{DocumentChild, TableCellContent, TableChild, TableRowChild};
    let docx =
        docx_rs::read_docx(bytes).map_err(|error| ParseError::ParseFailed(error.to_string()))?;
    let mut out = String::new();
    for child in docx.document.children {
        match child {
            DocumentChild::Paragraph(paragraph) => {
                let line = paragraph_text(&paragraph.children);
                // 标题段由段落样式 val 判断（Heading1..6），转为 Markdown 标题。
                let heading = paragraph
                    .property
                    .style
                    .as_ref()
                    .map(|style| style.val.as_str())
                    .and_then(heading_level);
                match heading {
                    Some(level) => out.push_str(&format!("{} {}\n\n", "#".repeat(level), line)),
                    None => {
                        if !line.trim().is_empty() {
                            out.push_str(line.trim());
                            out.push_str("\n\n");
                        }
                    }
                }
            }
            DocumentChild::Table(table) => {
                for row in table.rows {
                    let TableChild::TableRow(table_row) = row;
                    let cells = table_row
                        .cells
                        .iter()
                        .map(|cell| {
                            let TableRowChild::TableCell(table_cell) = cell;
                            let mut cell_text = String::new();
                            for content in &table_cell.children {
                                if let TableCellContent::Paragraph(paragraph) = content {
                                    cell_text.push_str(&paragraph_text(&paragraph.children));
                                }
                            }
                            cell_text.trim().to_string()
                        })
                        .collect::<Vec<_>>();
                    out.push_str(&format!("| {} |\n", cells.join(" | ")));
                }
                out.push('\n');
            }
            _ => {}
        }
    }
    let text = out.trim();
    if text.is_empty() {
        return Err(ParseError::ParseFailed("docx produced no text".into()));
    }
    Ok(text.to_string())
}

/// 提取段落的全部文本（含 run / hyperlink 内的文本）。
fn paragraph_text(children: &[docx_rs::ParagraphChild]) -> String {
    use docx_rs::{ParagraphChild, RunChild};
    let mut line = String::new();
    for child in children {
        match child {
            ParagraphChild::Run(run) => {
                for text in &run.children {
                    if let RunChild::Text(value) = text {
                        line.push_str(&value.text);
                    }
                }
            }
            ParagraphChild::Hyperlink(link) => {
                for run in &link.children {
                    if let ParagraphChild::Run(run) = run {
                        for text in &run.children {
                            if let RunChild::Text(value) = text {
                                line.push_str(&value.text);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    line
}

/// 把 docx 段落样式名映射为 Markdown 标题级别（Heading1..6 → 1..6）。
fn heading_level(style: &str) -> Option<usize> {
    match style {
        "Heading1" => Some(1),
        "Heading2" => Some(2),
        "Heading3" => Some(3),
        "Heading4" => Some(4),
        "Heading5" => Some(5),
        "Heading6" => Some(6),
        _ => None,
    }
}

async fn parse_html(bytes: &[u8]) -> Result<String, ParseError> {
    // 优先 Pandoc。
    if let Some(markdown) = parse_via_pandoc(bytes, "html").await {
        return Ok(markdown);
    }
    // 回退：html2md（CPU 密集，放 blocking 线程池）。
    let bytes = bytes.to_vec();
    let result = tokio::task::spawn_blocking(move || {
        let text = std::str::from_utf8(&bytes).map_err(|_| ParseError::NotUtf8)?;
        let markdown = html2md::parse_html(text);
        if markdown.trim().is_empty() {
            return Err(ParseError::ParseFailed("html produced no text".into()));
        }
        Ok(markdown.trim().to_string())
    })
    .await;
    match result {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(error)) => Err(error),
        Err(error) => Err(ParseError::ParseFailed(format!(
            "html parse task join: {error}"
        ))),
    }
}

fn parse_text(bytes: &[u8]) -> Result<String, ParseError> {
    let text = std::str::from_utf8(bytes).map_err(|_| ParseError::NotUtf8)?;
    Ok(text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn passes_through_markdown_and_text() {
        let md = "# 标题\n\n正文内容";
        assert_eq!(
            parse_to_markdown("doc.md", md.as_bytes())
                .await
                .expect("md"),
            "# 标题\n\n正文内容"
        );
        assert_eq!(
            parse_to_markdown("doc.txt", "纯文本".as_bytes())
                .await
                .expect("txt"),
            "纯文本"
        );
    }

    #[tokio::test]
    async fn rejects_unsupported_type() {
        let err = parse_to_markdown("sheet.xlsx", b"x")
            .await
            .expect_err("xlsx unsupported");
        assert!(matches!(err, ParseError::UnsupportedType(_)));
    }

    #[tokio::test]
    async fn rejects_empty_file() {
        let err = parse_to_markdown("doc.txt", b"")
            .await
            .expect_err("empty rejected");
        assert!(matches!(err, ParseError::Empty));
    }

    #[tokio::test]
    async fn rejects_invalid_utf8() {
        let err = parse_to_markdown("doc.txt", &[0xff, 0xfe, 0x00])
            .await
            .expect_err("bad utf8");
        assert!(matches!(err, ParseError::NotUtf8));
    }
}
