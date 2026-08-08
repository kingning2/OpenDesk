//! LLM plain-text → HTML email template generation.
//!
//! 作者：coisini
//! 创建时间：2026-08-01

/// System prompt for styled HTML email generation (email-agent parity).
pub const EMAIL_HTML_SYSTEM_PROMPT: &str = r#"你是一个专业的 HTML 邮件前端开发专家与 UI 设计师。

【我的需求】
接下来我会发送邮件纯文本内容。请将其转化为排版精美、格式统一的 HTML 模板。

【统一样式规范】
1. HTML 规范：必须使用严格的 HTML 4.01 Strict 标准，兼容 Outlook、Gmail、Apple Mail 等各类邮件客户端。布局一律使用 <table> 嵌套，全局禁止使用外链 CSS，所有样式必须内联（Inline Style）。
2. 配色与主视觉：
   - 页面背景：浅灰 #f9f9f9
   - 内容卡片：纯白背景 #ffffff，边框 1px solid #e0e0e0，圆角 4px，最大宽度 600px，居中显示
   - 主色调（按钮/主链接/序号）：品牌蓝色 #4a90e2
   - 主文字颜色：深灰 #333333，正文字号 15px，行高 1.6，字体 Arial, sans-serif
3. 特殊区块：填写项用浅灰背景；风险提示用浅红背景；优惠/退款方案用浅黄背景；操作步骤用蓝色序号列表；关键行动点使用 MSO 兼容的蓝色胶囊按钮。
4. body要使用overflow: hidden;这个css属性，来让滚动条禁用了。

【输出要求】
1. 修正输入文本中明显的英文拼写或语法小错误。
2. 直接输出完整的 HTML 4.01 代码，不要使用 markdown 代码块包裹。"#;

/// Parsed AI email HTML result.
///
/// 作者：coisini
/// 创建时间：2026-08-01
#[derive(Debug, Clone)]
pub struct GeneratedMailHtml {
    /// Render-ready HTML body.
    pub html: String,
    /// Optional trailing notes from the model.
    pub notes: Option<String>,
}

/// 使用 Rust LLM 客户端生成邮件 HTML。
pub struct GenerateMailHtml;

impl GenerateMailHtml {
    /// 将纯文本邮件交给 LLM，并解析为 HTML 与可选说明。
    ///
    /// # Errors
    ///
    /// 输入为空、LLM 请求失败或解析后 HTML 为空时返回错误。
    pub async fn execute(
        client: &agent::llm::LlmClient,
        text: &str,
    ) -> Result<GeneratedMailHtml, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("Email text is required".to_string());
        }
        let prompt = agent::prompt::Prompt::new(EMAIL_HTML_SYSTEM_PROMPT, text);
        let raw = client
            .complete(&prompt)
            .await
            .map_err(|error| error.to_string())?;
        let parsed = parse_email_html_response(&raw);
        if parsed.html.trim().is_empty() {
            return Err("AI returned empty email HTML".to_string());
        }
        Ok(parsed)
    }
}

/// Parse raw LLM output into HTML + optional notes.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub fn parse_email_html_response(raw_response: &str) -> GeneratedMailHtml {
    let mut without_fences = raw_response.trim().to_string();
    if let Some(stripped) = without_fences.strip_prefix("```html") {
        without_fences = stripped.to_string();
    } else if let Some(stripped) = without_fences.strip_prefix("```") {
        without_fences = stripped.to_string();
    }
    if let Some(stripped) = without_fences.strip_suffix("```") {
        without_fences = stripped.trim().to_string();
    }

    if let Some(index) = without_fences.find("---NOTES---") {
        let html = without_fences[..index].trim().to_string();
        let notes = without_fences[index + "---NOTES---".len()..]
            .trim()
            .to_string();
        return GeneratedMailHtml {
            html,
            notes: (!notes.is_empty()).then_some(notes),
        };
    }

    if let Some(html) = extract_html_document(&without_fences) {
        return GeneratedMailHtml { html, notes: None };
    }

    GeneratedMailHtml {
        html: without_fences,
        notes: None,
    }
}

fn extract_html_document(input: &str) -> Option<String> {
    let lower = input.to_ascii_lowercase();
    if let Some(start) = lower.find("<!doctype") {
        if let Some(end) = lower.rfind("</html>") {
            return Some(input[start..end + "</html>".len()].trim().to_string());
        }
    }
    if let Some(start) = lower.find("<html") {
        if let Some(end) = lower.rfind("</html>") {
            return Some(input[start..end + "</html>".len()].trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_email_html_response;

    #[test]
    fn strips_markdown_fence() {
        let parsed = parse_email_html_response("```html\n<html><body>Hi</body></html>\n```");
        assert!(parsed.html.contains("<html>"));
    }

    #[test]
    fn separates_optional_notes() {
        let parsed =
            parse_email_html_response("<html><body>Hi</body></html>\n---NOTES---\nReview CTA");
        assert_eq!(parsed.notes.as_deref(), Some("Review CTA"));
    }

    #[test]
    fn extracts_html_from_surrounding_model_text() {
        let parsed = parse_email_html_response(
            "Here is the result:\n<!DOCTYPE html><html><body>Hi</body></html>\nDone",
        );
        assert_eq!(parsed.html, "<!DOCTYPE html><html><body>Hi</body></html>");
    }
}
