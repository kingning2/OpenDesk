//! 把 Markdown 文本按语义边界切成适合向量化的块。
//!
//! 策略：先按空行把文本拆成「块单元」（标题 / 段落 / 列表 / 表格 / 引用 / 代码块 /
//! 分隔线），再按块单元聚合：标题行开启新块，标题仅含无正文时并入首个正文单元
//! 保证标题与内容不分离；正文单元累积到目标长度后收盘。
//! **绝不从单个单元中间切断**——超长段落、列表、代码块整体保留，保证语义完整。

/// 单块目标字符数（中文场景按字符计）。
const CHUNK_TARGET_CHARS: usize = 1200;

/// 把 Markdown 文本按语义边界切成适合向量化的块。
///
/// # 参数
/// - `md` — 解析后的 Markdown 文本
///
/// # 返回值
/// 非空内容块列表；无有效内容时返回空列表。纯标题块（无正文）会被丢弃。
pub fn chunk_markdown(md: &str) -> Vec<String> {
    let blocks = split_blocks(md);
    let units = blocks
        .iter()
        .map(|block| classify_block(block))
        .collect::<Vec<_>>();

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_chars = 0usize;
    // 当前块是否已含正文（仅标题时并入首个正文单元，避免标题与内容分离）。
    let mut current_has_body = false;

    for unit in units {
        let is_heading = matches!(unit.kind, UnitKind::Heading(_));
        let unit_chars = unit.text.chars().count();
        if is_heading {
            // 标题开启新块，保证标题与后续内容不分离。
            if !current.is_empty() {
                chunks.push(std::mem::take(&mut current));
            }
            current = unit.text;
            current_chars = unit_chars;
            current_has_body = false;
        } else if current.is_empty() {
            current = unit.text;
            current_chars = unit_chars;
            current_has_body = true;
        } else if current_chars + unit_chars > CHUNK_TARGET_CHARS && current_has_body {
            chunks.push(std::mem::take(&mut current));
            current = unit.text;
            current_chars = unit_chars;
            current_has_body = true;
        } else {
            append_to(&mut current, &mut current_chars, &unit.text);
            current_has_body = true;
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }

    // 丢弃纯标题块（无正文内容）。
    chunks.retain(|chunk| has_body(chunk));
    chunks
}

/// 一个语义块单元的类型。
#[derive(Debug, Clone, PartialEq, Eq)]
enum UnitKind {
    /// 标题，携带级别（1..=6）。
    Heading(usize),
    /// 普通段落。
    Paragraph,
    /// 列表（含缩进续行）。
    List,
    /// 引用块。
    Quote,
    /// 围栏代码块。
    Code,
    /// 表格。
    Table,
    /// 分隔线（`---` 等）。
    Rule,
}

/// 一个语义块单元：类型 + 原始文本。
struct Unit {
    kind: UnitKind,
    text: String,
}

/// 按空行把 Markdown 拆成「行块」，每个块是一段连续的非空行。
fn split_blocks(md: &str) -> Vec<Vec<&str>> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    for line in md.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks
}

/// 识别一个行块的类型并生成单元。
fn classify_block(lines: &[&str]) -> Unit {
    let first = lines[0].trim_start();

    // 围栏代码块。
    if first.starts_with("```") || first.starts_with("~~~") {
        return Unit {
            kind: UnitKind::Code,
            text: lines.join("\n"),
        };
    }
    // 单行标题。
    if lines.len() == 1 {
        if let Some(level) = heading_level(first) {
            return Unit {
                kind: UnitKind::Heading(level),
                text: first.to_string(),
            };
        }
    }
    // 单行分隔线。
    if lines.len() == 1 && is_rule_line(first) {
        return Unit {
            kind: UnitKind::Rule,
            text: first.to_string(),
        };
    }
    // 表格：块内包含分隔行（`|---|---|`）。
    if lines.iter().any(|line| looks_like_table_separator(line)) {
        return Unit {
            kind: UnitKind::Table,
            text: lines.join("\n"),
        };
    }
    // 引用块：每行都以 `>` 开头。
    if lines.iter().all(|line| line.trim_start().starts_with('>')) {
        return Unit {
            kind: UnitKind::Quote,
            text: lines.join("\n"),
        };
    }
    // 列表：每行都是列表项或缩进续行。
    if lines.iter().all(|line| is_list_line(line)) {
        return Unit {
            kind: UnitKind::List,
            text: lines.join("\n"),
        };
    }
    Unit {
        kind: UnitKind::Paragraph,
        text: lines.join("\n"),
    }
}

/// 提取标题级别；非标题返回 `None`（要求 `#` 后跟空格）。
fn heading_level(line: &str) -> Option<usize> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &line[hashes..];
    if rest.starts_with(' ') || rest.is_empty() {
        Some(hashes)
    } else {
        None
    }
}

/// 单行分隔线：整行由 `-` / `*` / `_` 组成且不少于 3 个字符。
fn is_rule_line(line: &str) -> bool {
    let t = line.trim();
    if t.chars().count() < 3 {
        return false;
    }
    let mark = t.chars().next().unwrap_or(' ');
    mark == '-' || mark == '*' || mark == '_'
}

/// 表格分隔行：以 `|` 开头且包含 `---`。
fn looks_like_table_separator(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('|') && t.contains("---")
}

/// 列表项起始：`- ` / `* ` / `+ `，或数字后跟 `.` / `)`。
fn is_list_start(line: &str) -> bool {
    let t = line.trim_start();
    if t.starts_with("- ") || t.starts_with("* ") || t.starts_with("+ ") {
        return true;
    }
    let digits = t.bytes().take_while(|byte| byte.is_ascii_digit()).count();
    if digits > 0 {
        let rest = &t[digits..];
        if rest.starts_with(". ") || rest.starts_with(") ") {
            return true;
        }
    }
    false
}

/// 列表行：列表项起始，或以空白缩进的续行。
fn is_list_line(line: &str) -> bool {
    is_list_start(line) || line.starts_with(' ') || line.starts_with('\t')
}

/// 把文本追加到当前块（块间以空行分隔），并刷新字符数。
fn append_to(current: &mut String, chars: &mut usize, text: &str) {
    if current.is_empty() {
        current.push_str(text);
    } else {
        current.push_str("\n\n");
        current.push_str(text);
    }
    *chars = current.chars().count();
}

/// 块是否包含非标题的正文行（用于丢弃纯标题块）。
fn has_body(chunk: &str) -> bool {
    chunk.lines().any(|line| {
        let t = line.trim_start();
        !t.is_empty() && !t.starts_with('#')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_by_heading() {
        let md = "## 概述\n\n这是第一部分。\n\n### 细节\n\n这是第二部分。";
        let chunks = chunk_markdown(md);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].contains("概述"));
        assert!(chunks[1].contains("细节"));
    }

    #[test]
    fn drops_empty_sections() {
        let md = "## 只有标题\n\n# 另一个\n\n正文内容";
        let chunks = chunk_markdown(md);
        assert!(!chunks.is_empty());
        assert!(chunks.iter().any(|c| c.contains("正文内容")));
    }

    #[test]
    fn no_heading_treats_whole_as_one() {
        let chunks = chunk_markdown("一段没有标题的文本。");
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn keeps_huge_paragraph_whole() {
        // 语义切块：单个超长段落不从中切断。
        let long = "字".repeat(2500);
        let md = format!("## 长文\n\n{long}");
        let chunks = chunk_markdown(&md);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].contains(&long));
    }

    #[test]
    fn splits_between_paragraphs_at_semantic_boundary() {
        // 两个 800 字段落超过目标长度，应在段落边界切，不在段落中间切。
        let para_a = "甲".repeat(800);
        let para_b = "乙".repeat(800);
        let md = format!("## 标题\n\n{para_a}\n\n{para_b}");
        let chunks = chunk_markdown(&md);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].contains('甲') && !chunks[0].contains('乙'));
        assert!(chunks[1].contains('乙') && !chunks[1].contains('甲'));
    }

    #[test]
    fn keeps_list_together() {
        let md = "- 苹果\n- 香蕉\n- 橙子";
        let chunks = chunk_markdown(md);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].contains("香蕉"));
    }

    #[test]
    fn keeps_code_block_whole() {
        let md = "```rust\nfn main() {}\n```";
        let chunks = chunk_markdown(md);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].contains("fn main"));
    }

    #[test]
    fn drops_heading_only_first_chunk() {
        // 文档标题（#）后无正文时，纯标题块被丢弃。
        let md = "# 公司产品手册\n\n## 第一章\n\n正文1\n\n## 第二章\n\n正文2";
        let chunks = chunk_markdown(md);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].contains("第一章"));
        assert!(chunks[1].contains("第二章"));
    }
}
