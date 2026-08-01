//! 采集关键词的 LLM prompt、执行与响应解析。

use std::collections::HashSet;

use serde_json::Value;

/// 关键词生成的系统提示词。
pub const KEYWORDS_SYSTEM_PROMPT: &str = "You generate YouTube search keywords for B2B lead discovery. Return ONLY a JSON array of strings. No markdown, no explanation.";

/// 构造多语言关键词生成提示词。
pub fn build_keywords_user_prompt(
    directions: &[String],
    languages: &[String],
    count_per_language: usize,
    requested: usize,
) -> String {
    format!(
        "Keyword directions (themes), comma-separated:\n{}\n\n\
         Target languages (generate keywords written for searching in each language):\n{}\n\n\
         Exactly {count_per_language} keywords for EACH language.\n\
         Total expected keywords: {requested}.\n\
         Make keywords concrete, searchable on YouTube, and aligned with the directions.\n\
         Avoid duplicates across languages when the same spelling would collide.",
        directions.join(", "),
        languages.join(", ")
    )
}

/// 将模型输出解析为去重、限长的关键词列表。
pub fn parse_keyword_list(raw: &str, limit: usize) -> Vec<String> {
    let cleaned = strip_code_fence(raw.trim());
    let mut output = Vec::new();
    let mut seen = HashSet::new();

    if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(&cleaned) {
        for item in items {
            let text = match item {
                Value::String(value) => value,
                value => value.to_string().trim_matches('"').to_string(),
            };
            push_keyword(&mut output, &mut seen, &text, limit);
            if output.len() >= limit {
                break;
            }
        }
        return output;
    }

    for line in cleaned.lines() {
        push_keyword(&mut output, &mut seen, &clean_list_line(line), limit);
        if output.len() >= limit {
            break;
        }
    }
    output
}

/// 使用已配置 LLM 生成采集关键词。
pub struct GenerateCrawlerKeywords;

impl GenerateCrawlerKeywords {
    /// 生成并解析关键词，返回列表及期望总数。
    ///
    /// # Errors
    ///
    /// 方向、语言为空，LLM 请求失败或响应没有可用关键词时返回错误。
    pub async fn execute(
        client: &agent::llm::LlmClient,
        directions: &[String],
        languages: &[String],
        count_per_language: usize,
    ) -> Result<(Vec<String>, usize), String> {
        if directions.is_empty() {
            return Err("directions is required (comma-separated)".to_string());
        }
        if languages.is_empty() {
            return Err("languages is required (comma-separated)".to_string());
        }
        let count_per_language = count_per_language.clamp(1, 200);
        let requested = count_per_language.saturating_mul(languages.len());
        let user_prompt =
            build_keywords_user_prompt(directions, languages, count_per_language, requested);
        let prompt =
            agent::prompt::Prompt::new(KEYWORDS_SYSTEM_PROMPT, &user_prompt).with_temperature(0.4);
        let raw = client
            .complete(&prompt)
            .await
            .map_err(|error| error.to_string())?;
        let keywords = parse_keyword_list(&raw, requested);
        if keywords.is_empty() {
            return Err("LLM returned no usable keywords".to_string());
        }
        Ok((keywords, requested))
    }
}

fn strip_code_fence(raw: &str) -> String {
    let mut cleaned = raw.trim();
    if let Some(value) = cleaned.strip_prefix("```json") {
        cleaned = value;
    } else if let Some(value) = cleaned.strip_prefix("```JSON") {
        cleaned = value;
    } else if let Some(value) = cleaned.strip_prefix("```") {
        cleaned = value;
    }
    cleaned
        .trim()
        .strip_suffix("```")
        .unwrap_or(cleaned.trim())
        .trim()
        .to_string()
}

fn clean_list_line(line: &str) -> String {
    let mut text = line.trim();
    for prefix in ["- ", "* ", "• "] {
        if let Some(value) = text.strip_prefix(prefix) {
            text = value.trim();
            break;
        }
    }
    for separator in ['.', ')'] {
        if let Some(index) = text.find(separator) {
            if index > 0
                && text[..index]
                    .chars()
                    .all(|character| character.is_ascii_digit())
            {
                return text[index + separator.len_utf8()..].trim().to_string();
            }
        }
    }
    text.to_string()
}

fn push_keyword(output: &mut Vec<String>, seen: &mut HashSet<String>, text: &str, limit: usize) {
    let text = text.trim().trim_matches('"').trim();
    if text.is_empty() || text.chars().count() > 255 || output.len() >= limit {
        return;
    }
    if seen.insert(text.to_lowercase()) {
        output.push(text.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::parse_keyword_list;

    #[test]
    fn parses_fenced_json_and_deduplicates_case_insensitively() {
        let parsed = parse_keyword_list(
            "```json\n[\"Rust CRM\", \"rust crm\", \"AI sales\"]\n```",
            5,
        );
        assert_eq!(parsed, vec!["Rust CRM", "AI sales"]);
    }

    #[test]
    fn falls_back_to_numbered_lines_and_honors_limit() {
        let parsed = parse_keyword_list("1. first\n2) second\n- third", 2);
        assert_eq!(parsed, vec!["first", "second"]);
    }

    #[test]
    fn ignores_empty_and_oversized_keywords() {
        let oversized = "x".repeat(256);
        let parsed = parse_keyword_list(&format!("[\"\", \"{oversized}\", \"valid\"]"), 5);
        assert_eq!(parsed, vec!["valid"]);
    }
}
